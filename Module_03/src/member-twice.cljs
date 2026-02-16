#_{:clj-kondo/ignore [:namespace-name-mismatch]}
(ns member-twice)
(defn member-twice?
  [x coll]
  (let [after-first (rest (drop-while #(not= x %) coll))]
    (some #(= x %) after-first)))