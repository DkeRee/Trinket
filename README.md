![Alt Text](https://media.discordapp.net/attachments/950533914760458340/1025279005596856361/image0.gif)

# Trinket
Named after a hilarious spongebob meme, Trinket, a UCI Negamax engine, is here to clap and be clapped.
I was dragged into this rabbit hole by 101Donutman, and I do not regret it one bit.
I aspire to get Trinket to 3000 elo one day, but only time will tell for that ;)

## Features

### Search
- PV Search w/ Negamax
- Tranposition Table
- Quiescence Search
- Reductions
  - LMR (Late Move Reduction)
- Pruning
  - Basic Alpha-Beta
  - NMP (Null Move Pruning)
  - RFP (Reverse Futility Pruning)
  - Negative loud moves in QSearch found with SEE

### Move Ordering
- Hash Move
- Loud Moves
  - SEE
- Quiet Moves
  - History Heuristic
  - Killer Moves
  - Some other smaller checks like Castling bonus...etc

### Static Evaluation
- Trained with Texel Tuning with the data set lichess-big3-resolved
- Piece Weights
- Piece Square Tables
- Tempo
- Bishop Pair Bonus
- Passed Pawn Bonus
- Pawn Island Penalty
- Pawn Isolation Penalty
- Rook on Open File Bonus
- Rook on Semi-Open File Bonus

### Time Management
- Calculates the time spent on searching based on the given wtime/btime and winc/binc (Time and Increment)
- If the time limit is exceeded during a search, it will abort and return the last best move from the prior depth

## Shoutouts
- MinusKelvin for being MinusEleven, I mean Seven Eleven? What
- Analog Hors. Pony.
- Pali
- Andrew (GM Andy?)
- KingKP
- Malarksist
- Shak kar shack car s shak shrek ?? h
