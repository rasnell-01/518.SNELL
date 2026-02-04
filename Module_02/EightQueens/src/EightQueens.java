import java.util.*;
import static java.lang.System.out;

public class EightQueens {
    private final List<List<String>> solutions;
    private final boolean[] columns;
    private final boolean[] diagonals;
    private final boolean[] antiDiagonals;
    private final int[] queens;  
    
    public EightQueens() {
        solutions = new ArrayList<>();
        columns = new boolean[8];
        diagonals = new boolean[15];      
        antiDiagonals = new boolean[15];  
        queens = new int[8];
        Arrays.fill(queens, -1);
    }

    public List<List<String>> solveNQueens() {
        backtrack(0);
        return solutions;
    }

    private void backtrack(int row) {
        if (row == 8) {
            solutions.add(constructSolution());
            return;
        }
        
        for (int col = 0; col < 8; col++) {
            int diag = row - col + 7;      
            int antiDiag = row + col;
            
            if (!columns[col] && !diagonals[diag] && !antiDiagonals[antiDiag]) {
                queens[row] = col;
                columns[col] = true;
                diagonals[diag] = true;
                antiDiagonals[antiDiag] = true;
                
                backtrack(row + 1);
                
                queens[row] = -1;
                columns[col] = false;
                diagonals[diag] = false;
                antiDiagonals[antiDiag] = false;
            }
        }
    }

    private List<String> constructSolution() {
        List<String> board = new ArrayList<>();
        for (int row = 0; row < 8; row++) {
            StringBuilder sb = new StringBuilder();
            for (int col = 0; col < 8; col++) {
                if (queens[row] == col) {
                    sb.append('Q');
                } else {
                    sb.append('.');
                }
            }
            board.add(sb.toString());
        }
        return board;
    }

    private void printSolution(List<String> solution) {
        for (String row : solution) {
            out.println(row);
        }
        out.println();
    }

    public static void main(String[] args) {
        EightQueens solver = new EightQueens();
        List<List<String>> solutions = solver.solveNQueens();

        out.println("Total solutions found: " + solutions.size());
        out.println("\nFirst solution:");
        solver.printSolution(solutions.getFirst());

        out.println("All solutions:");
        for (int i = 0; i < solutions.size(); i++) {
            out.println("Solution " + (i + 1) + ":");
            solver.printSolution(solutions.get(i));
        }
    }
}