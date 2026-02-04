import static java.lang.System.out;
import java.util.Random;

public class BowlingSimulator {
    public static void main(String[] args) {
        Game game = new Game();
        Random rand = new Random();
        for (int frame = 0; frame < 10; frame++) {
            int firstRoll = rand.nextInt(11);
            game.roll(firstRoll);

            if (firstRoll == 10) {
                if (frame == 9) {
                    int bonusRoll = rand.nextInt(11);
                    game.roll(bonusRoll);

                    int bonusRoll2 = (bonusRoll == 10) ? rand.nextInt(11) : rand.nextInt(11 - bonusRoll);
                    game.roll(bonusRoll2);
                }
            }

        }
        out.println(game.score());
    }
}
