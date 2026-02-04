# AlternateEightQueens was a project to solve the 8 queens puzzle using backtracking. 
# THIS WAS DONE IN CLUADE! I coppied and pasted the question 2.
        ## Key Concepts Explained:

        **1. Diagonal Calculation:**
        - **Diagonal** (↘): For squares on the same diagonal, `row - col` is constant
  - Example: (0,0), (1,1), (2,2) all have row - col = 0
        - We add 7 to make indices positive: 0 to 14

        - **Anti-diagonal** (↙): For squares on the same anti-diagonal, `row + col` is constant
  - Example: (0,7), (1,6), (2,5) all have row + col = 7
        - Range: 0 to 14

        **2. Backtracking Steps:**
        1. Place queen at (row, col)
2. Mark column, diagonal, anti-diagonal as used
3. Recurse to next row
4. If successful, solution is found
5. Unmark everything (backtrack) to try other positions

**Output:**

Total solutions found: 92

Solution 1:
Q.......
....Q...
.......Q
.....Q..
..Q.....
......Q.
.Q......
...Q....
