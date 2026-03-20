(ns todo-store.core
  (:import [java.util.concurrent Executors CountDownLatch]))

;; ═══════════════════════════════════════════════════════════════════════════
;; Events  (plain Clojure maps — immutable by default)
;; ═══════════════════════════════════════════════════════════════════════════

(defn add-item-event    [id text]     {:type :add-item     :id id :text text})
(defn complete-event    [id]          {:type :complete-item :id id})
(defn rename-event      [id new-text] {:type :rename-item   :id id :new-text new-text})
(defn delete-event      [id]          {:type :delete-item   :id id})

;; ═══════════════════════════════════════════════════════════════════════════
;; Store  — {:events [e0 e1 e2 …]}
;;
;; "Updating" means conj-ing onto the events vector, which returns a *new*
;; vector via Clojure's persistent data structures.  The old vector (and
;; therefore the old store) is completely unaffected.  Structural sharing
;; means only the new path through the internal 32-ary trie is allocated;
;; all existing nodes are shared with the previous version.
;; ═══════════════════════════════════════════════════════════════════════════

(def empty-store {:events []})

(defn append-event
  "Pure function: returns a NEW store with `event` appended.
   The caller's reference to `store` still points to the original persistent
   vector — no mutation, no copies of unchanged data."
  [store event]
  (update store :events conj event))

(defn event-count [store] (count (:events store)))

;; ═══════════════════════════════════════════════════════════════════════════
;; State machine  —  fold events onto an empty map
;; ═══════════════════════════════════════════════════════════════════════════

(def initial-state {})    ;; id -> {:text "…" :completed? false}

(defmulti ^:private apply-event
  "Dispatch on event type keyword."
  (fn [_state event] (:type event)))

(defmethod apply-event :add-item
  [state {:keys [id text]}]
  (assoc state id {:text text :completed? false}))

(defmethod apply-event :complete-item
  [state {:keys [id]}]
  (assoc-in state [id :completed?] true))

(defmethod apply-event :rename-item
  [state {:keys [id new-text]}]
  (assoc-in state [id :text] new-text))

(defmethod apply-event :delete-item
  [state {:keys [id]}]
  (dissoc state id))

(defmethod apply-event :default [state _event] state)

;; ═══════════════════════════════════════════════════════════════════════════
;; Snapshot API
;; ═══════════════════════════════════════════════════════════════════════════

(defn snapshot-at
  "Replay the first `v` events in `store`.
   subvec is O(1) — it returns a view into the persistent vector, not a copy."
  [store v]
  {:pre [(<= 0 v (event-count store))]}
  (reduce apply-event initial-state (subvec (:events store) 0 v)))

(defn current-snapshot
  "Replay all events."
  [store]
  (snapshot-at store (event-count store)))

(defn history
  "Return a vector of [version state] pairs for every version 0…N."
  [store]
  (mapv (fn [v] [v (snapshot-at store v)])
        (range (inc (event-count store)))))

(defn history-every-n
  "Return snapshots at versions 0, n, 2n, … — useful for large logs."
  [store n]
  (let [total (event-count store)]
    (mapv (fn [v] {:version v :state (snapshot-at store v)})
          (range 0 (inc total) n))))

;; ═══════════════════════════════════════════════════════════════════════════
;; Stretch — memoised checkpoints via an atom-backed cache
;;
;; The atom IS the only mutation here, and it is purely a performance cache.
;; The *values* it stores are still immutable maps; the public API remains
;; referentially transparent.  This illustrates that Clojure's atom is
;; "controlled mutation" — it never breaks the immutability of the data.
;; ═══════════════════════════════════════════════════════════════════════════

(defn make-checkpoint-store
  "Returns a store that caches computed snapshots in an atom.
   Multiple versions may share the same cache atom safely."
  []
  {:events []
   :cache  (atom {0 initial-state})})

(defn- cached-snapshot [{:keys [events cache]} v]
  (or (get @cache v)
      (let [best-v (apply max (filter #(<= % v) (keys @cache)))
            base   (get @cache best-v)
            result (reduce apply-event base (subvec events best-v v))]
        (swap! cache assoc v result)
        result)))

(defn append-event-cp
  "append-event for checkpoint stores; shares the cache atom across versions."
  [{:keys [events cache]} event]
  {:events (conj events event)
   :cache  cache})

(defn snapshot-at-cp
  "snapshot-at with lazy caching for checkpoint stores."
  [store v]
  {:pre [(<= 0 v (count (:events store)))]}
  (cached-snapshot store v))

;; ═══════════════════════════════════════════════════════════════════════════
;; Part A — Core Demo
;; ═══════════════════════════════════════════════════════════════════════════

(defn demo-part-a []
  (println "\n╔══════════════════════════════════════╗")
  (println "║     PART A: Core Demo                ║")
  (println "╚══════════════════════════════════════╝\n")

  ;; Build a chain of immutable store versions.
  ;; Each sN is a plain Clojure map.  No mutation occurs anywhere.
  (let [s0 empty-store
        s1 (append-event s0 (add-item-event    "a" "Buy milk"))
        s2 (append-event s1 (add-item-event    "b" "Write code"))
        s3 (append-event s2 (complete-event    "a"))
        s4 (append-event s3 (rename-event      "b" "Write better code"))
        s5 (append-event s4 (delete-event      "a"))]

    (println "── Full history (from s5 log) ──")
    (doseq [[v state] (history s5)]
      (println (format "  v%-2d %s" v state)))

    (println "\n── Sampled every 2 events ──")
    (doseq [{:keys [version state]} (history-every-n s5 2)]
      (println (format "  v%-2d %s" version state)))

    (println "\n── Time Travel ──")
    (println "  s0 current :" (current-snapshot s0))
    (println "  s2 current :" (current-snapshot s2))
    (println "  s5 current :" (current-snapshot s5))

    (println "\n── Older versions unchanged ──")
    (println "  s1 event count :" (event-count s1))
    (println "  s1 state       :" (current-snapshot s1))

    (println "\n── Assertions ──")
    (assert (= {} (current-snapshot s0))
            "s0 must be empty")
    (assert (= "Buy milk" (get-in (current-snapshot s1) ["a" :text]))
            "s1: item a must be 'Buy milk'")
    (assert (true? (get-in (current-snapshot s3) ["a" :completed?]))
            "s3: item a must be complete")
    ;; Key time-travel check: s1's snapshot == s5's log rewound to v1
    (assert (= (current-snapshot s1) (snapshot-at s5 1))
            "snapshot-at(s5,1) must match s1 current state")
    (assert (nil? (get (current-snapshot s5) "a"))
            "s5: deleted item a must be absent")
    (println "  All assertions passed ✓")))

;; ═══════════════════════════════════════════════════════════════════════════
;; Part B — Concurrency Experiment
;;
;; Design:
;;   - An atom holds the "currently published store" (an immutable value).
;;   - The WRITER calls swap! to atomically advance the published version.
;;     swap! only touches the *reference*; the store value itself never
;;     changes.
;;   - READERS deref the atom to capture the current immutable store, then
;;     call snapshot-at freely — NO locks needed because the captured value
;;     can never change under them.
;;   - A reader holding an old store reference remains valid forever; it
;;     simply sees an earlier (but still correct) slice of history.
;; ═══════════════════════════════════════════════════════════════════════════

(defn demo-part-b []
  (println "\n╔══════════════════════════════════════╗")
  (println "║     PART B: Concurrency Demo         ║")
  (println "╚══════════════════════════════════════╝\n")

  (let [published (atom empty-store)   ; latest immutable store published
        n-events  15
        n-readers 4
        latch     (CountDownLatch. 1)
        errors    (atom [])
        pool      (Executors/newFixedThreadPool (+ 1 n-readers))

        writer-fn
        (fn []
          (.await latch)
          (dotimes [i n-events]
            (swap! published
                   #(append-event % (add-item-event (str "item-" i)
                                                    (str "Task " i))))
            (Thread/sleep 8))
          (println (str "  [writer] done. published "
                        (event-count @published) " events.")))

        make-reader
        (fn [rid]
          (fn []
            (.await latch)
            (dotimes [iter (* n-events 5)]
              ;; Deref = grab the current immutable store — one coordinated step.
              ;; Everything after is pure reads on an immutable value.
              (let [store  @published
                    n      (event-count store)
                    v      (if (zero? n) 0 (mod iter (inc n)))
                    state  (snapshot-at store v)
                    ;; Invariant: every todo must have non-empty :text
                    bad    (filter (fn [[_ item]] (empty? (:text item))) state)]
                (when (seq bad)
                  (swap! errors conj {:reader rid :v v :bad bad}))))
            (println (str "  [reader-" rid "] done."))))]

    (let [wf (.submit pool ^Callable writer-fn)
          rfs (mapv #(.submit pool ^Callable (make-reader %)) (range n-readers))]
      (.countDown latch)
      (.get wf)
      (doseq [f rfs] (.get f)))
    (.shutdown pool)

    (if (empty? @errors)
      (println "\n  Concurrency test passed — zero torn reads ✓")
      (do (println "\n  ERRORS detected:")
          (doseq [e @errors] (println "   " e))))))

;; ═══════════════════════════════════════════════════════════════════════════
;; Tests
;; ═══════════════════════════════════════════════════════════════════════════

(defn run-tests []
  (println "\n╔══════════════════════════════════════╗")
  (println "║     TESTS                            ║")
  (println "╚══════════════════════════════════════╝\n")

  ;; T1: Old version unchanged after subsequent appends
  (let [s0 empty-store
        s1 (append-event s0 (add-item-event "x" "hello"))
        s2 (append-event s1 (complete-event "x"))]
    (assert (= {} (snapshot-at s0 0))
            "T1a FAIL: s0 v0 should be empty")
    (assert (= "hello" (get-in (snapshot-at s1 1) ["x" :text]))
            "T1b FAIL: s1 v1 should contain item x")
    (assert (true? (get-in (snapshot-at s2 2) ["x" :completed?]))
            "T1c FAIL: s2 v2 should mark x complete")
    (println "  T1 (old version unchanged)     PASSED ✓"))

  ;; T2: Non-interference — building s2 must not change s1
  (let [s0 empty-store
        s1 (append-event s0 (add-item-event "a" "Task A"))
        snap-before (snapshot-at s1 1)
        _s2 (append-event s1 (add-item-event "b" "Task B"))
        snap-after  (snapshot-at s1 1)]
    (assert (= snap-before snap-after)
            "T2 FAIL: s1 snapshot changed after building s2")
    (println "  T2 (non-interference)          PASSED ✓"))

  ;; T3: snapshot-at v=0 is always empty-state
  (let [big (reduce (fn [s i] (append-event s (add-item-event (str i) (str i))))
                    empty-store (range 10))]
    (assert (= {} (snapshot-at big 0))
            "T3 FAIL: v0 should always be empty-state")
    (println "  T3 (v0 always empty)           PASSED ✓"))

  ;; T4: delete removes item
  (let [s (-> empty-store
              (append-event (add-item-event "a" "A"))
              (append-event (delete-event "a")))]
    (assert (nil? (get (current-snapshot s) "a"))
            "T4 FAIL: deleted item still present")
    (println "  T4 (delete)                    PASSED ✓"))

  ;; T5: rename changes text
  (let [s (-> empty-store
              (append-event (add-item-event "a" "Old"))
              (append-event (rename-event "a" "New")))]
    (assert (= "New" (get-in (current-snapshot s) ["a" :text]))
            "T5 FAIL: rename did not change text")
    (println "  T5 (rename)                    PASSED ✓"))

  ;; T6: time travel — snapshot-at(full-store, v) == state of that generation
  (let [s0 empty-store
        s1 (append-event s0 (add-item-event "a" "A"))
        s2 (append-event s1 (complete-event "a"))
        s3 (append-event s2 (delete-event "a"))]
    (assert (= (current-snapshot s1) (snapshot-at s3 1))
            "T6a FAIL: time-travel to v1 failed")
    (assert (= (current-snapshot s2) (snapshot-at s3 2))
            "T6b FAIL: time-travel to v2 failed")
    (println "  T6 (time-travel)               PASSED ✓"))

  ;; T7: append-event returns a new value, not the same reference
  (let [s0 empty-store
        s1 (append-event s0 (add-item-event "a" "A"))]
    (assert (not (identical? s0 s1))
            "T7 FAIL: append must produce a new value")
    (assert (= 0 (event-count s0))
            "T7 FAIL: s0 event count changed")
    (assert (= 1 (event-count s1))
            "T7 FAIL: s1 event count wrong")
    (println "  T7 (new value produced)        PASSED ✓"))

  ;; T8: checkpoint store produces same results as plain store
  (let [events [(add-item-event "a" "A")
                (add-item-event "b" "B")
                (complete-event "a")
                (rename-event "b" "BB")
                (delete-event "a")]
        plain  (reduce append-event    empty-store events)
        cp     (reduce append-event-cp (make-checkpoint-store) events)]
    (doseq [v (range (inc (count events)))]
      (let [plain-state (snapshot-at    plain v)
            cp-state    (snapshot-at-cp cp    v)]
        (assert (= plain-state cp-state)
                (str "T8 FAIL: mismatch at v=" v))))
    (println "  T8 (checkpoint == plain)       PASSED ✓"))

  (println "\n  All tests passed ✓"))

;; ═══════════════════════════════════════════════════════════════════════════
;; Entry Point
;; ═══════════════════════════════════════════════════════════════════════════

(defn -main [& _args]
  (run-tests)
  (demo-part-a)
  (demo-part-b))
