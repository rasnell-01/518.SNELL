(ns TAT)

;; Helper functions to access tree components
(defn tat-value [tat] (first tat))
(defn tat-left [tat] (second tat))
(defn tat-right [tat] (nth tat 2))

;; Predicate to test if tree is empty
(defn tat-empty?
  "Returns true if the tree of atoms is empty"
  [tat]
  (or (nil? tat) (empty? tat)))

;; Insert an atom into the tree (BST property maintained)
(defn insert
  "Insert an atom into a tree of atoms, maintaining BST ordering"
  [tat atom]
  (cond
    (tat-empty? tat) (list atom '() '())         ; Create new leaf node
    (< atom (tat-value tat))                      ; Go left
      (list (tat-value tat)
            (insert (tat-left tat) atom)
            (tat-right tat))
    (> atom (tat-value tat))                      ; Go right
      (list (tat-value tat)
            (tat-left tat)
            (insert (tat-right tat) atom))
    :else tat))                                   ; Duplicate, no insert (or could allow)

;; Check if an atom is in the tree
(defn member?
  "Returns true if atom is in the tree of atoms"
  [tat atom]
  (cond
    (tat-empty? tat) false
    (= atom (tat-value tat)) true
    (< atom (tat-value tat)) (member? (tat-left tat) atom)
    :else (member? (tat-right tat) atom)))

;; Helper: find minimum value in a tree
(defn find-min
  "Find the minimum value in a non-empty tree"
  [tat]
  (if (tat-empty? (tat-left tat))
    (tat-value tat)
    (find-min (tat-left tat))))

;; Delete an atom from the tree
(defn delete
  "Delete the first occurrence of an atom from the tree"
  [tat atom]
  (cond
    (tat-empty? tat) '()                          ; Not found, return empty
    (< atom (tat-value tat))                      ; Go left
      (list (tat-value tat)
            (delete (tat-left tat) atom)
            (tat-right tat))
    (> atom (tat-value tat))                      ; Go right
      (list (tat-value tat)
            (tat-left tat)
            (delete (tat-right tat) atom))
    :else                                          ; Found the node to delete
      (cond
        (tat-empty? (tat-left tat)) (tat-right tat)   ; No left child
        (tat-empty? (tat-right tat)) (tat-left tat)   ; No right child
        :else                                          ; Two children
          (let [successor (find-min (tat-right tat))]
            (list successor
                  (tat-left tat)
                  (delete (tat-right tat) successor))))))

;; In-order traversal: left, root, right
(defn in-order
  "Traverse tree in-order, applying exp to each node, returning list of results"
  [tat exp]
  (if (tat-empty? tat)
    '()
    (concat (in-order (tat-left tat) exp)
            (list (exp (tat-value tat)))
            (in-order (tat-right tat) exp))))

;; Pre-order traversal: root, left, right
(defn pre-order
  "Traverse tree pre-order, applying exp to each node, returning list of results"
  [tat exp]
  (if (tat-empty? tat)
    '()
    (concat (list (exp (tat-value tat)))
            (pre-order (tat-left tat) exp)
            (pre-order (tat-right tat) exp))))

;; Post-order traversal: left, right, root
(defn post-order
  "Traverse tree post-order, applying exp to each node, returning list of results"
  [tat exp]
  (if (tat-empty? tat)
    '()
    (concat (post-order (tat-left tat) exp)
            (post-order (tat-right tat) exp)
            (list (exp (tat-value tat))))))