# Chess Engine 
This is my Chess Engine written in Rust. I started creating this engine as a little side project to learn the Rust programming language, but it has quickly spiraled out of control and grown into something much bigger than what I was expecting. Still, it isn't an advanced engine by any means, but I'm keeping on improving it piece by piece.

# User Interface
This engine uses a basic subset of the UCI protocol. If you want to play against it just download any UCI GUI and connect it to the engine. 

# Features 
- Bitboard board representation
- Hyperbola Quintessence move generation for sliding pieces
- Basic Transposition Table with Zobrist keys
- Alpha-Beta search with iterative deepening
- Very basic position evaluation with material counting
- Very basic move ordering with Hash Move, MVV-LVA, and promotions priority