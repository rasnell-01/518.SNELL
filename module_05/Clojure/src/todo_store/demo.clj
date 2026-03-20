(ns todo-store.demo
  (:require [todo-store.core :as core]
            [clojure.pprint :refer [pprint]]))

;; ═══════════════════════════════════════════════════════════════════════════
;; Part A — Time Travel Demo
;; ═══════════════════════════════════════════════════════════════════════════

(defn run-part-a []
  (println "\n╔══════════════════════════════════════════════════╗")
  (println   "║         Part A — Time Travel Demo               ║")
  (println   "╚══════════════════════════════════════════════════╝\n")

  ;; Build a chain of independent store versions.
  ;; Each binding is a separate, usable store — none is invalidated.
  (let [s0 core/empty-store
        s1 (core/append-event s0 (core/add-item-event 1 "Buy milk"))
        s2 (core/append-event s1 (core/add-item-event 2 "Write code"))
        s3 (core/append-event s2 (core/complete-event 1))
        s4 (core/append-event s3 (core/rename-event 2 "Write better code"))
        s5 (core/append-event s4 (core/delete-event 1))]

    (println "Store event counts: s0=" (core/event-count s0)
             " s1=" (core/event-count s1)
             " s2=" (core/event-count s2)
             " s5=" (core/event-count s5))

    (println "\n--- Snapshots via s5 (all events present) ---")
    (doseq [v (range 6)]
      (printf "  v%-2d → %s%n" v (core/snapshot-at s5 v)))

    (println "\n--- Proving old stores are UNCHANGED ---")
    (printf "  snapshot-at(s0, 0)  = %s   [expect {}]%n"
            (core/snapshot-at s0 0))
    (printf "  snapshot-at(s1, 1)  = %s   [still has only Add milk]%n"
            (core/snapshot-at s1 1))
    (printf "  snapshot-at(s2, 2)  = %s   [still pre-complete]%n"
            (core/snapshot-at s2 2))

    (println "\n--- history-every-n on s5, step=2 ---")
    (pprint (core/history-every-n s5 2))

    ;; Return the stores so the REPL caller can inspect them
    {:s0 s0 :s1 s1 :s2 s2 :s3 s3 :s4 s4 :s5 s5}))

;; ═══════════════════════════════════════════════════════════════════════════
;; Part B — Concurrency Demo
;;
;; Design:
;;   atom `latest-store` is the single publication point for the "current"
;;   store version.  It is updated by one writer using swap! (compare-and-swap,
;;   no explicit lock).
;;
;;   Each reader dereferences the atom to get a *snapshot of the reference*
;;   — an immutable store value — then calls snapshot-at on that local value.
;;   Because the store is immutable, the reader never needs a lock and never
;;   sees a partially-written state.
;; ═══════════════════════════════════════════════════════════════════════════

(def ^:private scenario-events
  [(core/add-item-event    1 "Task A")
   (core/add-item-event    2 "Task B")
   (core/complete-event    1)
   (core/add-item-event    3 "Task C")
   (core/rename-event      2 "Task B (renamed)")
   (core/delete-event      1)
   (core/add-item-event    4 "Task D")
   (core/complete-event    3)])

(defn run-part-b []
  (println "\n╔══════════════════════════════════════════════════╗")
  (println   "║       Part B — Concurrency Demo                 ║")
  (println   "╚══════════════════════════════════════════════════╝\n")

  (let [latest-store (atom core/empty-store)   ;; publication point
        errors       (atom [])
        iterations   30
        num-readers  5

        ;; ── Writer ──────────────────────────────────────────────────────
        ;; swap! passes the *current* atom value to the function and
        ;; atomically installs the return value.  If two threads swap!
        ;; concurrently, one will win and the other will retry — no data loss.
        writer
        (future
          (doseq [e scenario-events]
            (swap! latest-store #(core/append-event % e))
            (Thread/sleep 15))
          (println "  [writer] done — published"
                   (core/event-count @latest-store) "events"))

        ;; ── Readers ─────────────────────────────────────────────────────
        ;; Each reader captures a *value* from the atom; that value is
        ;; forever immutable so no further synchronisation is needed.
        readers
        (doall
          (map
            (fn [rid]
              (future
                (dotimes [_ iterations]
                  (let [store @latest-store          ;; O(1) pointer read
                        v     (core/event-count store)
                        snap  (core/snapshot-at store v)]  ;; lock-free
                    ;; Sanity: every item in snap must have a non-nil text
                    (doseq [[id item] snap]
                      (when (nil? (:text item))
                        (swap! errors conj
                               {:reader rid :id id :issue :nil-text}))))
                  (Thread/sleep 5))
                (println (format "  [reader %d] done" rid))))
            (range num-readers)))]

    ;; Wait for all threads
    @writer
    (run! deref readers)

    (println)
    (if (empty? @errors)
      (println (format "  ✓ All %d read operations completed — zero errors, no torn state."
                       (* num-readers iterations)))
      (do (println "  ✗ Errors detected:")
          (pprint @errors)))

    ;; Show final state for inspection
    (println "\n  Final snapshot (current):")
    (pprint (core/current-snapshot @latest-store))))

;; ═══════════════════════════════════════════════════════════════════════════
;; Entry point
;; ═══════════════════════════════════════════════════════════════════════════

(defn -main [& _args]
  (run-part-a)
  (run-part-b)
  (shutdown-agents))
