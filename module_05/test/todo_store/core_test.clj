(ns todo-store.core-test
  (:require [clojure.test :refer [deftest is testing run-tests]]
            [todo-store.core :as core]))

;; ═══════════════════════════════════════════════════════════════════════════
;; Helper
;; ═══════════════════════════════════════════════════════════════════════════

(defn- build-store
  "Build a store by appending each event in `events`."
  [events]
  (reduce core/append-event core/empty-store events))

;; ═══════════════════════════════════════════════════════════════════════════
;; 1. Immutability — append-event never mutates the old store
;; ═══════════════════════════════════════════════════════════════════════════

(deftest test-append-does-not-mutate-old-store
  (testing "s0 has 0 events after creating s1"
    (let [s0 core/empty-store
          s1 (core/append-event s0 (core/add-item-event 1 "Buy milk"))]
      (is (= 0 (core/event-count s0)))
      (is (= 1 (core/event-count s1)))))

  (testing "s1 has 1 event after creating s2"
    (let [s0 core/empty-store
          s1 (core/append-event s0 (core/add-item-event 1 "A"))
          s2 (core/append-event s1 (core/add-item-event 2 "B"))]
      (is (= 1 (core/event-count s1)))
      (is (= 2 (core/event-count s2))))))

;; ═══════════════════════════════════════════════════════════════════════════
;; 2. snapshot-at correctness
;; ═══════════════════════════════════════════════════════════════════════════

(deftest test-snapshot-at-empty
  (is (= {} (core/snapshot-at core/empty-store 0))))

(deftest test-snapshot-at-add-item
  (let [store (build-store [(core/add-item-event 1 "Buy milk")])]
    (is (= {} (core/snapshot-at store 0)))
    (is (= {1 {:text "Buy milk" :completed? false}}
           (core/snapshot-at store 1)))))

(deftest test-snapshot-at-complete-item
  (let [store (build-store [(core/add-item-event 1 "Buy milk")
                            (core/complete-event 1)])]
    (is (false? (get-in (core/snapshot-at store 1) [1 :completed?])))
    (is (true?  (get-in (core/snapshot-at store 2) [1 :completed?])))))

(deftest test-snapshot-at-rename-item
  (let [store (build-store [(core/add-item-event 1 "old")
                            (core/rename-event 1 "new")])]
    (is (= "old" (get-in (core/snapshot-at store 1) [1 :text])))
    (is (= "new" (get-in (core/snapshot-at store 2) [1 :text])))))

(deftest test-snapshot-at-delete-item
  (let [store (build-store [(core/add-item-event 1 "temp")
                            (core/delete-event 1)])]
    (is (contains? (core/snapshot-at store 1) 1))
    (is (not (contains? (core/snapshot-at store 2) 1)))))

;; ═══════════════════════════════════════════════════════════════════════════
;; 3. Time-travel / non-interference
;; ═══════════════════════════════════════════════════════════════════════════

(deftest test-time-travel-non-interference
  (testing "snapshot-at(s1,1) unchanged after creating s2"
    (let [s1          (core/append-event core/empty-store
                                         (core/add-item-event 1 "Task"))
          snap-before (core/snapshot-at s1 1)
          _s2         (core/append-event s1 (core/complete-event 1))
          snap-after  (core/snapshot-at s1 1)]
      (is (= snap-before snap-after))
      (is (false? (:completed? (get snap-before 1))))))

  (testing "earlier version never sees later events"
    (let [s0 core/empty-store
          s1 (core/append-event s0 (core/add-item-event 1 "A"))
          s3 (-> s1
                 (core/append-event (core/add-item-event 2 "B"))
                 (core/append-event (core/complete-event 1)))]
      ;; s0 snapshot is still empty regardless of s3
      (is (= {} (core/current-snapshot s0)))
      ;; s1 snapshot still shows only item 1 un-completed
      (is (= {1 {:text "A" :completed? false}}
             (core/current-snapshot s1)))
      ;; s3 sees everything
      (is (= {1 {:text "A" :completed? true}
              2 {:text "B" :completed? false}}
             (core/current-snapshot s3))))))

;; ═══════════════════════════════════════════════════════════════════════════
;; 4. history
;; ═══════════════════════════════════════════════════════════════════════════

(deftest test-history-length
  (let [store (build-store (map #(core/add-item-event % (str "item-" %))
                                (range 5)))]
    (is (= 6 (count (core/history store))))))   ;; versions 0..5

(deftest test-history-every-n
  (let [store (build-store (map #(core/add-item-event % "x") (range 10)))]
    (let [h (core/history-every-n store 3)]
      (is (= [0 3 6 9] (mapv :version h))))))

;; ═══════════════════════════════════════════════════════════════════════════
;; 5. Concurrency sanity
;;    Writer pumps events; multiple readers snapshot older versions.
;;    Assert: no exceptions, no nil :text values in any snapshot.
;; ═══════════════════════════════════════════════════════════════════════════

(deftest test-concurrent-readers-writers
  (let [latest (atom core/empty-store)
        errors (atom [])
        n-events  20
        n-readers 5
        n-iters   50

        writer
        (future
          (dotimes [i n-events]
            (swap! latest #(core/append-event
                            % (core/add-item-event (inc i) (str "item-" i))))
            (Thread/sleep 5)))

        readers
        (doall
          (map (fn [rid]
                 (future
                   (dotimes [_ n-iters]
                     (let [store @latest
                           v     (core/event-count store)]
                       (try
                         (let [snap (core/snapshot-at store v)]
                           (doseq [[id item] snap]
                             (when (nil? (:text item))
                               (swap! errors conj {:reader rid :id id}))))
                         (catch Exception e
                           (swap! errors conj {:reader rid :ex (str e)}))))
                     (Thread/sleep 2))))
               (range n-readers)))]

    @writer
    (run! deref readers)
    (is (empty? @errors)
        (str "Concurrency errors: " @errors))))

;; ═══════════════════════════════════════════════════════════════════════════
;; 6. Checkpoint store
;; ═══════════════════════════════════════════════════════════════════════════

(deftest test-checkpoint-store
  (let [store (reduce core/append-event-cp
                      (core/make-checkpoint-store)
                      [(core/add-item-event 1 "A")
                       (core/add-item-event 2 "B")
                       (core/complete-event 1)])]
    (is (= {} (core/snapshot-at-cp store 0)))
    (is (= {1 {:text "A" :completed? false}}
           (core/snapshot-at-cp store 1)))
    (is (true? (get-in (core/snapshot-at-cp store 3) [1 :completed?])))))

;; ═══════════════════════════════════════════════════════════════════════════
;; Runner (for direct invocation)
;; ═══════════════════════════════════════════════════════════════════════════

(comment
  (run-tests 'todo-store.core-test))
