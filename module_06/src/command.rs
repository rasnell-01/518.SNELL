#[derive(Debug, Clone)]
pub enum Command {
    /// Swap the elements at positions `i` and `j`.
    Swap(usize, usize),
}

pub fn insertion_sort_plan(slice: &[i32]) -> Vec<Command> {
    let mut sim = slice.to_vec();
    let mut commands = Vec::new();

    for i in 1..sim.len() {
        let mut j = i;
        while j > 0 && sim[j - 1] > sim[j] {
            commands.push(Command::Swap(j - 1, j));
            sim.swap(j - 1, j); // keep simulation consistent
            j -= 1;
        }
    }

    commands
}

pub fn interpret(commands: &[Command], data: &mut Vec<i32>) {
    for cmd in commands {
        match cmd {
            Command::Swap(i, j) => data.swap(*i, *j),
        }
    }
}

pub fn insertion_sort(data: &mut Vec<i32>) {
    let commands = insertion_sort_plan(data); // pure
    interpret(&commands, data);               // impure
}
