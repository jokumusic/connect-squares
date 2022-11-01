import * as anchor from '@project-serum/anchor';
import { AnchorError, Program } from '@project-serum/anchor';
import {PublicKey, Keypair, sendAndConfirmTransaction} from "@solana/web3.js";
import { TicTacToe } from '../target/types/tic_tac_toe';
import chai from 'chai';
import chaiAsPromised from 'chai-as-promised';
import { expect } from 'chai';


async function play(program: Program<TicTacToe>, game, pot, player, tile, expectedMoves, expectedCurrentPlayerIndex, expectedGameState, expectedBoard) {
  console.log('marking tile: ', tile);
  const tx = await program.methods
    .gamePlay(tile)
    .accounts({
      player: player.publicKey,
      game,
      pot
    })
    .transaction();

  const txSignature = await anchor.web3.sendAndConfirmTransaction(program.provider.connection, tx, [player], {commitment: 'finalized'});
  const txConfirmation = await program.provider.connection
        .confirmTransaction(txSignature,'finalized');
  
  const gameState = await program.account.game.fetch(game);
  expect(gameState.moves).to.equal(expectedMoves);
  expect(gameState.currentPlayerIndex).to.equal(expectedCurrentPlayerIndex);
  expect(gameState.state).to.eql(expectedGameState);
  expect(gameState.board).to.eql(expectedBoard);
}

describe('tic-tac-toe', () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env())
  const program = anchor.workspace.TicTacToe as Program<TicTacToe>;
  const provider = program.provider as anchor.AnchorProvider;
  const playerOne = anchor.web3.Keypair.generate();
  const playerTwo = anchor.web3.Keypair.generate();
  const gameNonce = Math.floor((Math.random() * Math.pow(2,32)));
  const [gamePda, gamePdaBump] = PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("game"),
      playerOne.publicKey.toBuffer(),
      new anchor.BN(gameNonce).toArrayLike(Buffer, 'be', 4),
    ], program.programId);
  const [potPda, potPdaBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      anchor.utils.bytes.utf8.encode("pot"),
      gamePda.toBuffer()
    ], program.programId); 
  
  
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

  it('setup game!', async() => {
    const rows = 3;
    const cols = 3;
    const minPlayers = 2;
    const maxPlayers = 2;
    const wager = .001;

    const tx = await program.methods
      .gameInit(gameNonce,rows,cols, minPlayers,maxPlayers, wager)
      .accounts({
        creator: playerOne.publicKey,
        game: gamePda,
        pot: potPda,     
      })
      .transaction();
    
    const txSignature = await anchor.web3.sendAndConfirmTransaction(provider.connection, tx, [playerOne], {commitment: 'finalized'});
    const txConfirmation = await provider.connection.confirmTransaction(txSignature,'finalized');
        

    let game = await program.account.game.fetch(gamePda);
    expect(game.bump).to.equal(gamePdaBump);
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
  });

  it('player two joins game', async () => {
    const tx = await program.methods
    .gameJoin()
    .accounts({
      player: playerTwo.publicKey,
      game: gamePda,
      pot: potPda,  
    })
    .transaction();
  
    const txSignature = await anchor.web3.sendAndConfirmTransaction(provider.connection, tx, [playerTwo], {commitment: 'finalized'});
    const txConfirmation = await provider.connection.confirmTransaction(txSignature,'finalized');

    let game = await program.account.game.fetch(gamePda);
    expect(game.state).to.eql({ active:{} });
    expect(game.players).to.eql([playerOne.publicKey, playerTwo.publicKey]);
  });

  it('player one wins!', async () => {    
    
    const tx = await play(
      program,
      gamePda,
      potPda,
      playerOne,
      {row: 0, column: 0},
      1,
      1,
      { active: {}, },
      [
        [0,null,null],
        [null,null,null],
        [null,null,null]
      ]
    );

    try {
      await play(
        program,
        gamePda,
        potPda,
        playerOne, // same player in subsequent turns
        // change sth about the tx because
        // duplicate tx that come in too fast
        // after each other may get dropped
        {row: 1, column: 0},
        1,
        1,
        { active: {}, },
        [
          [0,null,null],
          [null,null,null],
          [null,null,null]
        ]
      );
      chai.assert(false, "should've failed but didn't ");
    } catch (_err) {
      //expect(_err).to.be.instanceOf(AnchorError);
      //const err: AnchorError = _err;
      //expect(err.error.errorCode.code).to.equal("NotPlayersTurn");
      //expect(err.error.errorCode.number).to.equal(6003);
      //expect(err.program.equals(program.programId)).is.true;
      //expect(err.error.comparedValues).to.deep.equal([playerTwo.publicKey, playerOne.publicKey]);
    }

    await play(
      program,
      gamePda,
      potPda,
      playerTwo,
      {row: 1, column: 0},
      2,
      0,
      { active: {}, },
      [
        [0,null,null],
        [1,null,null],
        [null,null,null]
      ]
    );

    await play(
      program,
      gamePda,
      potPda,
      playerOne,
      {row: 0, column: 1},
      3,
      1,
      { active: {}, },
      [
        [0,0,null],
        [1,null,null],
        [null,null,null]
      ]
    );

    try {
      await play(
        program,
        gamePda,
        potPda,
        playerTwo,
        {row: 5, column: 1}, // out of bounds row
        3,
        1,
        { active: {}, },
        [
          [0,0,null],
          [1,null,null],
          [null,null,null]
        ]
      );
      chai.assert(false, "should've failed but didn't ");
    } catch (_err) {
      //expect(_err).to.be.instanceOf(AnchorError);
      ///const err: AnchorError = _err;
      //expect(err.error.errorCode.number).to.equal(6000);
      //expect(err.error.errorCode.code).to.equal("TileOutOfBounds");
    }

    await play(
      program,
      gamePda,
      potPda,
      playerTwo,
      {row: 1, column: 1},
      4,
      0,
      { active: {}, },
      [
        [0,0,null],
        [1,1,null],
        [null,null,null]
      ]
    );

    try {
      await play(
        program,
        gamePda,
        potPda,
        playerOne,
        {row: 1, column: 1},
        4,
        0,
        { active: {}, },
        [
          [0,0,null],
          [1,1,null],
          [null,null,null]
        ]
      );
      chai.assert(false, "should've failed but didn't ");
    } catch (_err) {
      //expect(_err).to.be.instanceOf(AnchorError);
      //const err: AnchorError = _err;
      //expect(err.error.errorCode.number).to.equal(6001);
      //expect(err.error.errorCode.code).to.equal("TileAlreadySet");
    }

    await play(
      program,
      gamePda,
      potPda,
      playerOne,
      {row: 0, column: 2},
      5,
      0,//doesn't change player on winning move
      { won: { winner: playerOne.publicKey }, },
      [
        [0,0,0],
        [1,1,null],
        [null,null,null]
      ]
    );

    try {
      await play(
        program,
        gamePda,
        potPda,
        playerOne,
        {row: 0, column: 2},
        5,
        0,
        { won: { winner: playerOne.publicKey }, },
        [
          [0,0,0],
          [1,1,null],
          [null,null,null]
        ]
      );
      chai.assert(false, "should've failed but didn't ");
    } catch (_err) {
      //expect(_err).to.be.instanceOf(AnchorError);
      //const err: AnchorError = _err;
      //expect(err.error.errorCode.number).to.equal(6002);
      //expect(err.error.errorCode.code).to.equal("GameAlreadyOver");
    }
    
  });
/*
  it('tie', async () => {
    const gameKeypair = anchor.web3.Keypair.generate();
    const playerOne = programProvider.wallet;
    const playerTwo = anchor.web3.Keypair.generate();
    await program.methods
      .setupGame(playerTwo.publicKey)
      .accounts({
        game: gameKeypair.publicKey,
        playerOne: playerOne.publicKey,
      })
      .signers([gameKeypair])
      .rpc();

    let gameState = await program.account.game.fetch(gameKeypair.publicKey);
    expect(gameState.turn).to.equal(1);
    expect(gameState.players)
      .to
      .eql([playerOne.publicKey, playerTwo.publicKey]);
    expect(gameState.state).to.eql({ active: {} });
    expect(gameState.board)
      .to
      .eql([[null,null,null],[null,null,null],[null,null,null]]);

    await play(
      program,
      gameKeypair.publicKey,
      playerOne,
      {row: 0, column: 0},
      2,
      { active: {}, },
      [
        [{x:{}},null,null],
        [null,null,null],
        [null,null,null]
      ]
    );

    await play(
      program,
      gameKeypair.publicKey,
      playerTwo,
      {row: 1, column: 1},
      3,
      { active: {}, },
      [
        [{x:{}},null,null],
        [null,{o:{}},null],
        [null,null,null]
      ]
    );

    await play(
      program,
      gameKeypair.publicKey,
      playerOne,
      {row: 2, column: 0},
      4,
      { active: {}, },
      [
        [{x:{}},null,null],
        [null,{o:{}},null],
        [{x:{}},null,null]
      ]
    );

    await play(
      program,
      gameKeypair.publicKey,
      playerTwo,
      {row: 1, column: 0},
      5,
      { active: {}, },
      [
        [{x:{}},null,null],
        [{o:{}},{o:{}},null],
        [{x:{}},null,null]
      ]
    );

    await play(
      program,
      gameKeypair.publicKey,
      playerOne,
      {row: 1, column: 2},
      6,
      { active: {}, },
      [
        [{x:{}},null,null],
        [{o:{}},{o:{}},{x:{}}],
        [{x:{}},null,null]
      ]
    );

    await play(
      program,
      gameKeypair.publicKey,
      playerTwo,
      {row: 0, column: 1},
      7,
      { active: {}, },
      [
        [{x:{}},{o:{}},null],
        [{o:{}},{o:{}},{x:{}}],
        [{x:{}},null,null]
      ]
    );

    await play(
      program,
      gameKeypair.publicKey,
      playerOne,
      {row: 2, column: 1},
      8,
      { active: {}, },
      [
        [{x:{}},{o:{}},null],
        [{o:{}},{o:{}},{x:{}}],
        [{x:{}},{x:{}},null]
      ]
    );

    await play(
      program,
      gameKeypair.publicKey,
      playerTwo,
      {row: 2, column: 2},
      9,
      { active: {}, },
      [
        [{x:{}},{o:{}},null],
        [{o:{}},{o:{}},{x:{}}],
        [{x:{}},{x:{}},{o:{}}]
      ]
    );


    await play(
      program,
      gameKeypair.publicKey,
      playerOne,
      {row: 0, column: 2},
      9,
      { tie: {}, },
      [
        [{x:{}},{o:{}},{x:{}}],
        [{o:{}},{o:{}},{x:{}}],
        [{x:{}},{x:{}},{o:{}}]
      ]
    );
    
  })*/
});
