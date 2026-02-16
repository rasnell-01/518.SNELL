(ns Bidirectional-List)

(defn blist-empty
  "Create an empty bidirectional list"
  []
  '(nil () ()))

;; Create a bidirectional list from a regular list (cursor at start)
(defn blist-from-list
  "Create a bidirectional list from a regular list, cursor at first element"
  [lst]
  (if (empty? lst)
    (blist-empty)
    (list (first lst) '() (rest lst))))

;; Check if bidirectional list is empty
(defn blist-empty?
  "Check if the bidirectional list is empty"
  [blist]
  (and (nil? (first blist))
       (empty? (second blist))
       (empty? (nth blist 2))))

;; Get current element
(defn blist-current
  "Get the current element at cursor position"
  [blist]
  (first blist))

;; Move cursor forward
(defn blist-forward
  "Move cursor forward one position. Returns nil if at end."
  [blist]
  (let [current (first blist)
        before (second blist)
        after (nth blist 2)]
    (if (empty? after)
      nil                                          ; Can't move forward
      (list (first after)
            (if (nil? current) before (cons current before))
            (rest after)))))

;; Move cursor backward
(defn blist-backward
  "Move cursor backward one position. Returns nil if at beginning."
  [blist]
  (let [current (first blist)
        before (second blist)
        after (nth blist 2)]
    (if (empty? before)
      nil                                          ; Can't move backward
      (list (first before)
            (rest before)
            (if (nil? current) after (cons current after))))))

;; Insert element at current position (pushes current forward)
(defn blist-insert
  "Insert an element at current cursor position"
  [blist atom]
  (let [current (first blist)
        before (second blist)
        after (nth blist 2)]
    (list atom
          before
          (if (nil? current) after (cons current after)))))

;; Delete current element (cursor moves to next element)
(defn blist-delete
  "Delete element at current cursor position"
  [blist]
  (let [before (second blist)
        after (nth blist 2)]
    (if (empty? after)
      (if (empty? before)
        (blist-empty)
        (list (first before) (rest before) '()))
      (list (first after) before (rest after)))))

;; Convert back to regular list
(defn blist-to-list
  "Convert bidirectional list back to a regular list"
  [blist]
  (let [current (first blist)
        before (second blist)
        after (nth blist 2)]
    (concat (reverse before)
            (if (nil? current) '() (list current))
            after)))

;; Move to beginning
(defn blist-to-start
  "Move cursor to the beginning of the list"
  [blist]
  (blist-from-list (blist-to-list blist)))

;; Move to end
(defn blist-to-end
  "Move cursor to the end of the list"
  [blist]
  (loop [bl blist]
    (let [next (blist-forward bl)]
      (if (nil? next)
        bl
        (recur next)))))