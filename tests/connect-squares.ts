import * as anchor from '@project-serum/anchor';
import { AnchorError, Program } from '@project-serum/anchor';
import {PublicKey, Keypair, sendAndConfirmTransaction} from "@solana/web3.js";
import * as web3 from "@solana/web3.js";
import { ConnectSquares } from '../target/types/connect_squares';
import chai from 'chai';
import { expect } from 'chai';


export const GameState = {
  waiting:{waiting:{}},
  active:{active:{}},
  tie:{tie:{}},
  won:{won:{winner:{}}},
  cancelled:{cancelled:{}},
};

export type GameInitParameters = {
  gameNonce:number,
  gamePda:PublicKey,
  potPda: PublicKey,
  cols: number,
  rows: number,
  connect: number,
  minPlayers: number,
  maxPlayers: number,
  wager: number,
};

export type JoinGameParameters = {
  gamePda: PublicKey,
  potPda: PublicKey,
};

export type PlayParameters = {
  gamePda: PublicKey,
  potPda: PublicKey,
  tile: Tile,
};

export type Tile = {
  row: number,
  column: number,
}

export type ExpectedPlayResult = {
  moves: number,
  playerIndex: number,
  state: any,
  board: any,
}

async function getGamePda(program: Program<ConnectSquares>, creator: PublicKey, nonce?: number) : Promise<[PublicKey,number,number]> {

  if(!nonce) {
    do {
      nonce = Math.floor(Math.random() * Math.pow(2,32));
      const [pda, bump] = PublicKey.findProgramAddressSync(
        [Buffer.from("game"), creator.toBuffer(), new anchor.BN(nonce).toArrayLike(Buffer, 'be', 4)],
        program.programId
      );

      const existingGame = await program.account.game.fetchNullable(pda);
      if(!existingGame){
        return [pda,bump,nonce];
      }

    } while(true);
  }

  const [pda,bump] = PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("game"),
      creator.toBuffer(),
      new anchor.BN(nonce).toArrayLike(Buffer, 'be', 4),
    ], program.programId);
  
  return [pda,bump,nonce];
}

async function getPotPda(programId: PublicKey, gamePda: PublicKey) {
  return anchor.web3.PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("pot"),
      gamePda.toBuffer()
    ], programId);
}

async function getMetadataPda(programId: PublicKey){
  return anchor.web3.PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("metadata"),
    ], programId);
}

async function initGame(program: Program<ConnectSquares>, player: Keypair, params: GameInitParameters) {
  const tx = await program.methods
        .gameInit(params.gameNonce, params.rows, params.cols, params.connect, params.minPlayers, params.maxPlayers, params.wager)
        .accounts({
          creator: player.publicKey,
          game: params.gamePda,
          pot: params.potPda,     
        })
        .transaction();
      
  const txSignature = await anchor.web3.sendAndConfirmTransaction(program.provider.connection, tx, [player], {commitment: 'finalized'});
  const txConfirmation = await program.provider.connection.confirmTransaction(txSignature,'finalized');     
  return txConfirmation;
}

async function joinGame(program: Program<ConnectSquares>, player: Keypair, params: JoinGameParameters) {
  const tx = await program.methods
  .gameJoin()
  .accounts({
    player: player.publicKey,
    game: params.gamePda,
    pot: params.potPda,  
  })
  .transaction();

  const txSignature = await anchor.web3.sendAndConfirmTransaction(program.provider.connection, tx, [player], {commitment: 'finalized'});
  const txConfirmation = await program.provider.connection.confirmTransaction(txSignature,'finalized');

  return txConfirmation;
}

async function play(program: Program<ConnectSquares>, player: Keypair,  playParams: PlayParameters, expected: ExpectedPlayResult) {
  
  console.log('marking tile: ', playParams.tile);
  const [metadataPda] = await getMetadataPda(program.programId);

  const tx = await program.methods
    .gamePlay(playParams.tile)
    .accounts({
      player: player.publicKey,
      game: playParams.gamePda,
      pot: playParams.potPda,
      metadata: metadataPda,
    })
    .transaction();

  const txSignature = await anchor.web3.sendAndConfirmTransaction(program.provider.connection, tx, [player], {commitment: 'finalized'});
  const txConfirmation = await program.provider.connection
        .confirmTransaction(txSignature,'finalized');
  
  const game = await program.account.game.fetch(playParams.gamePda);
  //console.log(`game.currentPlayerIndex: ${game.currentPlayerIndex}, expected playerIndex: ${expected.playerIndex}, game.state: `, game.state);
  expect(game.moves).to.equal(expected.moves);  
  expect(game.currentPlayerIndex).to.equal(expected.playerIndex);
  expect(game.state).to.eql(expected.state);
  expect(game.board).to.eql(expected.board);

  return txConfirmation;
}

describe('connect-squares', () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env())
  const program = anchor.workspace.ConnectSquares as Program<ConnectSquares>;
  const provider = program.provider as anchor.AnchorProvider;
  const playerOne = anchor.web3.Keypair.generate();
  const playerTwo = anchor.web3.Keypair.generate();
  const wager = 100000;

  
  before(() => {
    return new Promise<void>(async (resolve,reject) => {
      console.log(`funding player accounts...`);
      
      const airdropSignature = await provider.connection
        .requestAirdrop(playerOne.publicKey, 40000000)
        .catch(reject);

      if(!airdropSignature)
        return;   

      const airdropConfirmation = await provider.connection
        .confirmTransaction(airdropSignature,'finalized')
        .catch(reject);

      if(!airdropConfirmation)
        return;

      const fundAccountsTx = new anchor.web3.Transaction();
      fundAccountsTx.add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: playerOne.publicKey,
          toPubkey: playerTwo.publicKey,
          lamports: 10000000
        })
      );      
    
      const fundAccountsTxSignature = await provider.connection.sendTransaction(fundAccountsTx, [playerOne]);
      const fundAccountsTxConfirmation = await provider.connection
        .confirmTransaction(fundAccountsTxSignature,'finalized')
        .catch(reject);

      resolve();
    });
  });

  
  it('init metadata', async() => {
    const [metadataPda, metadataPdaBump] = await getMetadataPda(program.programId);
    const tx = await program.methods
      .metadataInit()
      .accounts({
        authority: playerOne.publicKey,
        metadata: metadataPda,
      })
      .transaction();

      const txSignature = await anchor.web3.sendAndConfirmTransaction(program.provider.connection, tx, [playerOne], {commitment: 'finalized'});
      const txConfirmation = await program.provider.connection.confirmTransaction(txSignature,'finalized');

      const metadata = await program.account.metadata.fetch(metadataPda);
      expect(metadata.bump).to.equal(metadataPdaBump);
      expect(metadata.initialized).to.equal(true);
      expect(metadata.authority).to.eql(playerOne.publicKey);
  });

  it('re-init metadata fail - already initialized', async() => {
    const [metadataPda, metadataPdaBump] = await getMetadataPda(program.programId);
    const tx = await program.methods
      .metadataInit()
      .accounts({
        authority: playerOne.publicKey,
        metadata: metadataPda,
      })
      .transaction();

      try {
        const txSignature = await anchor.web3.sendAndConfirmTransaction(program.provider.connection, tx, [playerOne], {commitment: 'finalized'});
        const txConfirmation = await program.provider.connection.confirmTransaction(txSignature,'finalized');
        chai.assert(false, "should've failed but didn't ");
      } catch(_err) {
      }

    const metadata = await program.account.metadata.fetch(metadataPda);
    expect(metadata.bump).to.equal(metadataPdaBump);
    expect(metadata.initialized).to.equal(true);
    expect(metadata.authority).to.eql(playerOne.publicKey);
  });

  it('set metadata authority', async() => {
    const [metadataPda, metadataPdaBump] = await getMetadataPda(program.programId);
    const tx = await program.methods
      .metdataSetAuthority(provider.wallet.publicKey)
      .accounts({
        authority: playerOne.publicKey,
        metadata: metadataPda,
      })
      .transaction();

      const txSignature = await anchor.web3.sendAndConfirmTransaction(program.provider.connection, tx, [playerOne], {commitment: 'finalized'});
      const txConfirmation = await program.provider.connection.confirmTransaction(txSignature,'finalized');

      const metadata = await program.account.metadata.fetch(metadataPda);
      expect(metadata.bump).to.equal(metadataPdaBump);
      expect(metadata.initialized).to.equal(true);
      expect(metadata.authority).to.eql(provider.wallet.publicKey);
  });

  it('set metadata authority fail - invalid authority', async() => {
    const [metadataPda, metadataPdaBump] = await getMetadataPda(program.programId);
    const tx = await program.methods
      .metdataSetAuthority(playerOne.publicKey)
      .accounts({
        authority: playerOne.publicKey,
        metadata: metadataPda,
      })
      .transaction();

      try{
        const txSignature = await anchor.web3.sendAndConfirmTransaction(program.provider.connection, tx, [playerOne], {commitment: 'finalized'});
        const txConfirmation = await program.provider.connection.confirmTransaction(txSignature,'finalized');
        chai.assert(false, "should've failed but didn't ");
      }catch(_err) {
      }
  });

  it('metadata withdraw', async() =>{
    const [metadataPda, metadataPdaBump] = await getMetadataPda(program.programId);
    const lamports = 500000;

    const metadataBalanceBeforeFund = await program.provider.connection.getBalance(metadataPda);
    const fundMetadataTx = new anchor.web3
      .Transaction()
      .add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: program.provider.publicKey,
          toPubkey: metadataPda,
          lamports,
        })
      );

    const fundMetadataTxSignature = await program.provider.sendAndConfirm(fundMetadataTx)
    const metadataBalanceAfterFund = await provider.connection.getBalance(metadataPda);
    //console.log(`beforeFund: ${metadataBalanceBeforeFund}, afterFund: ${metadataBalanceAfterFund}`);
    expect(metadataBalanceAfterFund).is.equal(metadataBalanceBeforeFund + lamports);

    const withdrawTx = await program.methods
      .metadataWithdraw(new anchor.BN(lamports))
      .accounts({
        authority: program.provider.publicKey,
        metadata: metadataPda,
      })
      .transaction();

    const withdrawTxSignature = await program.provider.sendAndConfirm(withdrawTx);
    
    const metadataBalanceAfterWithdraw = await provider.connection.getBalance(metadataPda);
    //console.log("balance after withdraw: ", metadataBalanceAfterWithdraw);
    expect(metadataBalanceAfterWithdraw).is.equal(metadataBalanceBeforeFund);
  })

  it('metadata withdraw fail - unauthorized', async() =>{
    const [metadataPda, metadataPdaBump] = await getMetadataPda(program.programId);
    const lamports = 1;

    const metadataBalanceBeforeFund = await program.provider.connection.getBalance(metadataPda);
    const fundMetadataTx = new anchor.web3
      .Transaction()
      .add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: program.provider.publicKey,
          toPubkey: metadataPda,
          lamports,
        })
      );
      
    const fundMetadataTxSignature = await program.provider.sendAndConfirm(fundMetadataTx)
    const metadataBalanceAfterFund = await provider.connection.getBalance(metadataPda);    
    expect(metadataBalanceAfterFund).is.equal(metadataBalanceBeforeFund + lamports);

    const withdrawTx = await program.methods
      .metadataWithdraw(new anchor.BN(lamports))
      .accounts({
        authority: program.provider.publicKey,
        metadata: metadataPda,
      })
      .transaction();

    try{
      const withdrawTxSignature = await program.provider.sendAndConfirm(withdrawTx);
      chai.assert(false, "should've failed but didn't ");
    } catch(_err) {
    }  

  });


  
  it('setup game', async() => {
    const rows = 3;
    const cols = 3;
    const connect = 3;
    const minPlayers = 2;
    const maxPlayers = 2;

    const [gamePda, gamePdaBump, gameNonce] = await getGamePda(program, playerOne.publicKey);
    const [potPda, potPdaBump] = await getPotPda(program.programId, gamePda);

    const confirmation = await initGame(program, playerOne, {
      gameNonce:gameNonce,
      gamePda: gamePda,
      potPda: potPda,
      cols: cols,
      rows,
      connect,
      minPlayers,
      maxPlayers,
      wager,
    });
  
    let game = await program.account.game.fetch(gamePda);    
    expect(game.bump).to.equal(gamePdaBump);
    expect(game.version).to.be.greaterThanOrEqual(0);
    expect(game.moves).to.equal(0);
    expect(game.rows).to.equal(rows);
    expect(game.cols).to.equal(cols);
    expect(game.creator).to.eql(playerOne.publicKey);
    expect(game.currentPlayerIndex).to.equal(0);
    expect(game.initTimestamp.toNumber()).to.be.greaterThan(0);
    expect(game.lastMoveSlot.toNumber()).to.equal(0);
    expect(game.maxPlayers).to.equal(maxPlayers);
    expect(game.minPlayers).to.equal(minPlayers);
    expect(game.pot).to.eql(potPda);
    expect(game.players).to.eql([playerOne.publicKey, PublicKey.default]);
    expect(game.state).to.eql({ waiting:{} });
    expect(game.board)
      .to
      .eql([[null,null,null],[null,null,null],[null,null,null]]);

    let pot = await program.account.pot.fetch(potPda);
    expect(pot.bump).to.equal(potPdaBump);
    expect(pot.game).to.eql(gamePda);
  });

  it('join game', async () => {
    const rows = 3;
    const cols = 3;
    const connect = 3;
    const minPlayers = 2;
    const maxPlayers = 2;
    const [gamePda, gamePdaBump, gameNonce] = await getGamePda(program, playerOne.publicKey);
    const [potPda, potPdaBump] = await getPotPda(program.programId, gamePda);

    const initGameConfirmation = await initGame(program, playerOne, {
      gameNonce,
      gamePda: gamePda,
      potPda: potPda,
      cols: cols,
      rows,
      connect,
      minPlayers,
      maxPlayers,
      wager,
    });
    
    const confirmation = await joinGame(program, playerTwo, {gamePda, potPda});   

    let game = await program.account.game.fetch(gamePda);
    expect(game.state).to.eql({ active:{} });
    expect(game.players).to.have.deep.members([playerOne.publicKey, playerTwo.publicKey]);
  });

  it('cancel game', async() => {
    const rows = 3;
    const cols = 3;
    const connect = 3;
    const minPlayers = 2;
    const maxPlayers = 2;

    const [gamePda, gamePdaBump, gameNonce] = await getGamePda(program, playerOne.publicKey);
    const [potPda, potPdaBump] = await getPotPda(program.programId, gamePda);

    const confirmation = await initGame(program, playerOne, {
      gameNonce:gameNonce,
      gamePda: gamePda,
      potPda: potPda,
      cols: cols,
      rows,
      connect,
      minPlayers,
      maxPlayers,
      wager,
    });
  
    const balanceAfterInit = await program.provider.connection.getBalance(playerOne.publicKey);

    let game = await program.account.game.fetch(gamePda);
    expect(game.state).to.eql({ waiting:{} });

    const cancelTx = await program.methods
      .gameCancel()
      .accounts({
        player: playerOne.publicKey,
        game: gamePda,
        pot: potPda,
      })
      .transaction();

    const txSignature = await anchor.web3.sendAndConfirmTransaction(program.provider.connection, cancelTx, [playerOne], {commitment: 'finalized'});
    const txConfirmation = await program.provider.connection.confirmTransaction(txSignature,'finalized');

    const balanceAfterCancel = await program.provider.connection.getBalance(playerOne.publicKey);
    expect(balanceAfterCancel).to.be.greaterThan(balanceAfterInit);

    let updatedGame = await program.account.game.fetchNullable(gamePda);
    expect(updatedGame).to.be.null;
    let updatedPot = await program.account.pot.fetchNullable(potPda);
    expect(updatedPot).to.be.null;   
  });


  it('horizontal win!', async () => {
    const rows = 3;
    const cols = 3;
    const connect = 3;
    const minPlayers = 2;
    const maxPlayers = 2;
    let moves = 0;

    const [gamePda, gamePdaBump, gameNonce] = await getGamePda(program, playerOne.publicKey);
    const [potPda, potPdaBump] = await getPotPda(program.programId, gamePda);

    const initGameConfirmation = await initGame(program, playerOne, {
      gameNonce,
      gamePda: gamePda,
      potPda: potPda,
      cols: cols,
      rows,
      connect,
      minPlayers,
      maxPlayers,
      wager,
    });

    const joinGameConfirmation = await joinGame(program, playerTwo, { gamePda, potPda});
    const game = await program.account.game.fetch(gamePda);
    const players = game.players.map(p=>{ return p.equals(playerOne.publicKey) ? playerOne : playerTwo });
    let playerIndex = game.currentPlayerIndex;

    await play(program, players[playerIndex],
        { gamePda: gamePda, potPda: potPda, tile: {row: 0, column: 0}},
        { moves: ++moves, playerIndex: playerIndex ? --playerIndex : ++playerIndex, state: GameState.active, 
          board: [
            [0,null,null],
            [null,null,null],
            [null,null,null]
          ]
        }    
    );

    try {
      await play(program, 
        // same player in subsequent turns
        // change sth about the tx because
        // duplicate tx that come in too fast
        // after each other may get dropped
        players[playerIndex-1],
        { gamePda: gamePda, potPda: potPda, tile: {row: 1, column: 0}},
        { moves: moves, playerIndex: playerIndex, state: GameState.active, 
          board: [
            [0,null,null],
            [null,null,null],
            [null,null,null]
          ]
        }    
      );

      chai.assert(false, "should've failed but didn't ");
    } catch (_err) {
    }

    await play(program, players[playerIndex],
      { gamePda: gamePda, potPda: potPda, tile: {row: 1, column: 0}},
      { moves: ++moves, playerIndex: playerIndex ? --playerIndex : ++playerIndex, state: GameState.active, 
        board: [
          [0,null,null],
          [1,null,null],
          [null,null,null]
        ]
      }    
    );

    await play(program, players[playerIndex],
      { gamePda: gamePda, potPda: potPda, tile: {row: 0, column: 1}},
      { moves: ++moves, playerIndex: playerIndex ? --playerIndex : ++playerIndex, state: GameState.active, 
        board: [
          [0,0,null],
          [1,null,null],
          [null,null,null]
        ]
      }    
    );

    try {
      await play(program, players[playerIndex-1],
        { gamePda: gamePda, potPda: potPda, tile: {row: 5, column: 1}}, // out of bounds row
        { moves: moves, playerIndex: playerIndex, state: GameState.active, 
          board: [
            [0,0,null],
            [1,null,null],
            [null,null,null]
          ]
        }    
      );

      chai.assert(false, "should've failed but didn't ");
    } catch (_err) {
    }

    await play(program, players[playerIndex],
      { gamePda: gamePda, potPda: potPda, tile: {row: 1, column: 1}},
      { moves: ++moves, playerIndex: playerIndex ? --playerIndex : ++playerIndex, state: GameState.active, 
        board: [
          [0,0,null],
          [1,1,null],
          [null,null,null]
        ]
      }    
    );

    try {
      await play(program, players[playerIndex],
        { gamePda: gamePda, potPda: potPda, tile: {row: 1, column: 1}}, //should cause tile already set error
        { moves: moves, playerIndex: playerIndex, state: GameState.active, 
          board: [
            [0,0,null],
            [1,1,null],
            [null,null,null]
          ]
        }    
      );

      chai.assert(false, "should've failed but didn't ");
    } catch (_err) {
    }

    await play(program, players[playerIndex],
      { gamePda: gamePda, potPda: potPda, tile: {row: 0, column: 2}},
      { moves: ++moves, playerIndex: playerIndex, state: { won: { winner: players[playerIndex].publicKey }, }, 
        board: [
          [0,0,0],
          [1,1,null],
          [null,null,null]
        ]
      }    
    );
 

    try {
      await play(program, players[playerIndex],
        { gamePda: gamePda, potPda: potPda, tile: {row: 0, column: 2}}, //should throw an error that the game has already been won
        { moves: moves, playerIndex: playerIndex, state: { won: { winner: players[playerIndex].publicKey }, }, 
          board: [
            [0,0,0],
            [1,1,null],
            [null,null,null]
          ]
        }    
      );
      
      chai.assert(false, "should've failed but didn't ");
    } catch (_err) {
    }
    
  });

  it('vertical win!', async () => {
    const rows = 3;
    const cols = 3;
    const connect = 3;
    const minPlayers = 2;
    const maxPlayers = 2;
    let moves = 0;

    const [gamePda, gamePdaBump, gameNonce] = await getGamePda(program, playerOne.publicKey);
    const [potPda, potPdaBump] = await getPotPda(program.programId, gamePda);

    const initGameConfirmation = await initGame(program, playerOne, {
      gameNonce,
      gamePda: gamePda,
      potPda: potPda,
      cols: cols,
      rows,
      connect,
      minPlayers,
      maxPlayers,
      wager,
    });

    const joinGameConfirmation = await joinGame(program, playerTwo, { gamePda, potPda}); 
    const game = await program.account.game.fetch(gamePda);
    const players = game.players.map(p=>{ return p.equals(playerOne.publicKey) ? playerOne : playerTwo });
    let playerIndex = game.currentPlayerIndex;

    await play(program, players[playerIndex],
        { gamePda: gamePda, potPda: potPda, tile: {row: 0, column: 0}},
        { moves: ++moves, playerIndex: playerIndex ? --playerIndex : ++playerIndex, state: GameState.active, 
          board: [
            [0,null,null],
            [null,null,null],
            [null,null,null]
          ]
        }    
    );    
    
    await play(program, players[playerIndex],
      { gamePda: gamePda, potPda: potPda, tile: {row: 0, column: 1}},
      { moves: ++moves, playerIndex: playerIndex ? --playerIndex : ++playerIndex, state: GameState.active, 
        board: [
          [0,1,null],
          [null,null,null],
          [null,null,null]
        ]
      }    
    );

    await play(program, players[playerIndex],
      { gamePda: gamePda, potPda: potPda, tile: {row: 1, column: 0}},
      { moves: ++moves, playerIndex: playerIndex ? --playerIndex : ++playerIndex, state: GameState.active, 
        board: [
          [0,1,null],
          [0,null,null],
          [null,null,null]
        ]
      }    
    );
   
    await play(program, players[playerIndex],
      { gamePda: gamePda, potPda: potPda, tile: {row: 1, column: 1}},
      { moves: ++moves, playerIndex: playerIndex ? --playerIndex : ++playerIndex, state: GameState.active, 
        board: [
          [0,1,null],
          [0,1,null],
          [null,null,null]
        ]
      }    
    );

    await play(program, players[playerIndex],
      { gamePda: gamePda, potPda: potPda, tile: {row: 2, column:0}},
      { moves: ++moves, playerIndex: playerIndex, state: { won: { winner: players[playerIndex].publicKey }, }, 
        board: [
          [0,1,null],
          [0,1,null],
          [0,null,null]
        ]
      }    
    );
 
  });

  it('positive slope win!', async () => {
    const rows = 3;
    const cols = 3;
    const connect = 3;
    const minPlayers = 2;
    const maxPlayers = 2;
    let moves = 0;

    const [gamePda, gamePdaBump, gameNonce] = await getGamePda(program, playerOne.publicKey);
    const [potPda, potPdaBump] = await getPotPda(program.programId, gamePda);

    const initGameConfirmation = await initGame(program, playerOne, {
      gameNonce,
      gamePda: gamePda,
      potPda: potPda,
      cols: cols,
      rows,
      connect,
      minPlayers,
      maxPlayers,
      wager,
    });

    const joinGameConfirmation = await joinGame(program, playerTwo, { gamePda, potPda}); 
    const game = await program.account.game.fetch(gamePda);
    const players = game.players.map(p=>{ return p.equals(playerOne.publicKey) ? playerOne : playerTwo });
    let playerIndex = game.currentPlayerIndex;

    await play(program, players[playerIndex],
        { gamePda: gamePda, potPda: potPda, tile: {row: 2, column: 0}},
        { moves: ++moves, playerIndex: playerIndex ? --playerIndex : ++playerIndex, state: GameState.active, 
          board: [
            [null,null,null],
            [null,null,null],
            [0,null,null]
          ]
        }    
    );    
    
    await play(program, players[playerIndex],
      { gamePda: gamePda, potPda: potPda, tile: {row: 2, column: 1}},
      { moves: ++moves, playerIndex: playerIndex ? --playerIndex : ++playerIndex, state: GameState.active, 
        board: [
          [null,null,null],
          [null,null,null],
          [0,1,null]
        ]
      }    
    );


    await play(program, players[playerIndex],
      { gamePda: gamePda, potPda: potPda, tile: {row: 1, column: 1}},
      { moves: ++moves, playerIndex: playerIndex ? --playerIndex : ++playerIndex, state: GameState.active, 
        board: [
          [null,null,null],
          [null,0,null],
          [0,1,null]
        ]
      }    
    );

    await play(program, players[playerIndex],
      { gamePda: gamePda, potPda: potPda, tile: {row: 1, column: 0}},
      { moves: ++moves, playerIndex: playerIndex ? --playerIndex : ++playerIndex, state: GameState.active, 
        board: [
          [null,null,null],
          [1,0,null],
          [0,1,null]
        ]
      }    
    ); 

    await play(program, players[playerIndex],
      { gamePda: gamePda, potPda: potPda, tile: {row: 0, column:2}},
      { moves: ++moves, playerIndex: playerIndex, state: { won: { winner: players[playerIndex].publicKey }, }, 
        board: [
          [null,null,0],
          [1,0,null],
          [0,1,null]
        ]
      }    
    );

  });

  it('negative slope win!', async () => {
    const rows = 3;
    const cols = 3;
    const connect = 3;
    const minPlayers = 2;
    const maxPlayers = 2;
    let moves = 0;

    const [gamePda, gamePdaBump, gameNonce] = await getGamePda(program, playerOne.publicKey);
    const [potPda, potPdaBump] = await getPotPda(program.programId, gamePda);

    const initGameConfirmation = await initGame(program, playerOne, {
      gameNonce,
      gamePda: gamePda,
      potPda: potPda,
      cols: cols,
      rows,
      connect,
      minPlayers,
      maxPlayers,
      wager,
    });

    const joinGameConfirmation = await joinGame(program, playerTwo, { gamePda, potPda}); 
    const game = await program.account.game.fetch(gamePda);
    const players = game.players.map(p=>{ return p.equals(playerOne.publicKey) ? playerOne : playerTwo });
    let playerIndex = game.currentPlayerIndex;

    await play(program, players[playerIndex],
        { gamePda: gamePda, potPda: potPda, tile: {row: 0, column: 0}},
        { moves: ++moves, playerIndex: playerIndex ? --playerIndex : ++playerIndex, state: GameState.active, 
          board: [
            [0,null,null],
            [null,null,null],
            [null,null,null]
          ]
        }    
    );    
    
    await play(program, players[playerIndex],
      { gamePda: gamePda, potPda: potPda, tile: {row: 0, column: 1}},
      { moves: ++moves, playerIndex: playerIndex ? --playerIndex : ++playerIndex, state: GameState.active, 
        board: [
          [0,1,null],
          [null,null,null],
          [null,null,null]
        ]
      }    
    );

 
    await play(program, players[playerIndex],
      { gamePda: gamePda, potPda: potPda, tile: {row: 1, column: 1}},
      { moves: ++moves, playerIndex: playerIndex ? --playerIndex : ++playerIndex, state: GameState.active, 
        board: [
          [0,1,null],
          [null,0,null],
          [null,null,null]
        ]
      }    
    );

    await play(program, players[playerIndex],
      { gamePda: gamePda, potPda: potPda, tile: {row: 1, column: 0}},
      { moves: ++moves, playerIndex: playerIndex ? --playerIndex : ++playerIndex, state: GameState.active, 
        board: [
          [0,1,null],
          [1,0,null],
          [null,null,null]
        ]
      }    
    ); 

    await play(program, players[playerIndex],
      { gamePda: gamePda, potPda: potPda, tile: {row: 2, column:2}},
      { moves: ++moves, playerIndex: playerIndex, state: { won: { winner: players[playerIndex].publicKey }, }, 
        board: [
          [0,1,null],
          [1,0,null],
          [null,null,0]
        ]
      }    
    );
  });


});
