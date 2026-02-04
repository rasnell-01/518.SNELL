import java.util.*;

public class AlternateEightQueens {
    private List<List<String>> allSolutions;
    private boolean[] columnsUsed;
    private boolean[] diagonalsUsed;
    private boolean[] antiDiagonalsUsed;
    private int[] queenPositions;
    private static final int BOARD_SIZE = 8;

    public AlternateEightQueens() {
        allSolutions = new ArrayList<>();
        columnsUsed = new boolean[BOARD_SIZE];
        diagonalsUsed = new boolean[2 * BOARD_SIZE - 1];
        antiDiagonalsUsed = new boolean[2 * BOARD_SIZE - 1];
        queenPositions = new int[BOARD_SIZE];
        Arrays.fill(queenPositions, -1);
    }

    public List<List<String>> solve() {
        backtrack(0);
        return allSolutions;
    }

    /**
     * Backtracking function to place queens row by row
     * @param row Current row to place a queen
     */
    private void backtrack(int row) {
        // Base case: Successfully placed all 8 queens
        if (row == BOARD_SIZE) {
            allSolutions.add(createBoardRepresentation());
            return;
        }

        // Try placing queen in each column of the current row
        for (int col = 0; col < BOARD_SIZE; col++) {
            // Calculate diagonal indices
            // Diagonal (top-left to bottom-right): row - col is constant
            // We add (BOARD_SIZE - 1) to make the index non-negative
            int diagonalIndex = row - col + (BOARD_SIZE - 1);

            // Anti-diagonal (top-right to bottom-left): row + col is constant
            int antiDiagonalIndex = row + col;

            // Check if this position is safe (not under attack)
            if (isSafe(col, diagonalIndex, antiDiagonalIndex)) {
                // Place the queen
                placeQueen(row, col, diagonalIndex, antiDiagonalIndex);

                // Recursively place queens in the next row
                backtrack(row + 1);

                // Backtrack: Remove the queen and try next position
                removeQueen(row, col, diagonalIndex, antiDiagonalIndex);
            }
        }
    }

    private boolean isSafe(int col, int diag, int antiDiag) {
        return !columnsUsed[col] &&
                !diagonalsUsed[diag] &&
                !antiDiagonalsUsed[antiDiag];
    }

    private void placeQueen(int row, int col, int diag, int antiDiag) {
        queenPositions[row] = col;
        columnsUsed[col] = true;
        diagonalsUsed[diag] = true;
        antiDiagonalsUsed[antiDiag] = true;
    }

    private void removeQueen(int row, int col, int diag, int antiDiag) {
        queenPositions[row] = -1;
        columnsUsed[col] = false;
        diagonalsUsed[diag] = false;
        antiDiagonalsUsed[antiDiag] = false;
    }

    private List<String> createBoardRepresentation() {
        List<String> board = new ArrayList<>();
        for (int row = 0; row < BOARD_SIZE; row++) {
            char[] rowChars = new char[BOARD_SIZE];
            Arrays.fill(rowChars, '.');
            rowChars[queenPositions[row]] = 'Q';
            board.add(new String(rowChars));
        }
        return board;
    }

    public void printAllSolutions() {
        System.out.println("Total number of solutions: " + allSolutions.size());
        System.out.println();

        for (int i = 0; i < allSolutions.size(); i++) {
            System.out.println("Solution #" + (i + 1) + ":");
            printBoard(allSolutions.get(i));
        }
    }

    private void printBoard(List<String> board) {
        for (String row : board) {
            System.out.println(row);
        }
        System.out.println();
    }

    public static void main(String[] args) {
        AlternateEightQueens solver = new AlternateEightQueens();
        solver.solve();
        solver.printAllSolutions();
    }
}