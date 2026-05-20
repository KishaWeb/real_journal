## journal

ok so i made this journal cli for learnign rust cause i dont have any project ideas and i thought there is so many journal cli and it turned out i was right BUT alot of those journal were basic and quick with no tui i said ill make one with tui and quick commands so yeh i started making it, now so far im almost finished with quick command i lowky started coding since 4pm then took a lil break at 8:30 till 11 including a shower then cod'ed till 3:30 am now im going to watch a movie

## usage
ok i hadnt so far made it a full cli you still have to run it in cargo so here are the commands:
```bash
cargo run help
``` 
this shows the usage and commands
```bash
cargo run add <name> <description> <mood>
``` 
adds a journal (including the time it was made thats on your system)
```bash
cargo run list
``` 
shows the list of journals with their index's
```bash
cargo run show <name>
``` 
shows the journal with the name you put in
```bash
cargo run remove <index> , <index>...
``` 
removes the index you chose (dont chose the name of the journal)
```bash
cargo run edit <index> <new descriotion>
``` 
edits the requsted index name