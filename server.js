const WebSocket = require('ws');

// サーバーの設定
// Replitでは環境変数PORTが設定されているため使用
// ローカル環境では8080をデフォルトポートとして使用
const PORT = process.env.PORT || 8080;

// CORS対策を追加
const http = require('http');
const express = require('express');
const app = express();

// 接続しているクライアント一覧
const clients = new Map();
let nextPlayerId = 1;

// ゲーム状態
let gameState = {
  boardWidth: 16,
  boardHeight: 16,
  mineCount: 40,
  cells: [],
  revealed: [],
  flagged: [],
  gameStarted: false,
  gameOver: false,
  win: false
};

// ゲームの初期化
function initializeGame() {
  // セルの状態を初期化
  gameState.cells = Array(gameState.boardWidth * gameState.boardHeight).fill(0);
  gameState.revealed = Array(gameState.boardWidth * gameState.boardHeight).fill(false);
  gameState.flagged = Array(gameState.boardWidth * gameState.boardHeight).fill(false);
  gameState.gameStarted = false;
  gameState.gameOver = false;
  gameState.win = false;
}

// 地雷を配置
function placeMines(firstClickIndex) {
  const { boardWidth, boardHeight, mineCount } = gameState;
  const totalCells = boardWidth * boardHeight;
  
  // 最初にクリックしたセルとその周囲に地雷を配置しないようにする
  const excludedCells = getNeighbors(firstClickIndex);
  excludedCells.push(firstClickIndex);
  
  // 地雷をランダムに配置
  let minesPlaced = 0;
  while (minesPlaced < mineCount) {
    const randomIndex = Math.floor(Math.random() * totalCells);
    
    // 既に地雷がある場所や除外セルには配置しない
    if (gameState.cells[randomIndex] !== -1 && !excludedCells.includes(randomIndex)) {
      gameState.cells[randomIndex] = -1; // -1 は地雷を表す
      minesPlaced++;
      
      // 周囲のセルのカウントを増やす
      const neighbors = getNeighbors(randomIndex);
      for (const neighbor of neighbors) {
        if (gameState.cells[neighbor] !== -1) {
          gameState.cells[neighbor]++;
        }
      }
    }
  }
}

// 指定されたインデックスの周囲のセルを取得
function getNeighbors(index) {
  const { boardWidth, boardHeight } = gameState;
  const x = index % boardWidth;
  const y = Math.floor(index / boardWidth);
  
  const neighbors = [];
  
  for (let dy = -1; dy <= 1; dy++) {
    for (let dx = -1; dx <= 1; dx++) {
      if (dx === 0 && dy === 0) continue;
      
      const newX = x + dx;
      const newY = y + dy;
      
      if (newX >= 0 && newX < boardWidth && newY >= 0 && newY < boardHeight) {
        neighbors.push(newY * boardWidth + newX);
      }
    }
  }
  
  return neighbors;
}

// セルを開く
function revealCell(index) {
  const { boardWidth, boardHeight, cells, revealed, flagged, gameOver } = gameState;
  
  // 既に開かれている、フラグが立てられている、またはゲームオーバーの場合は何もしない
  if (revealed[index] || flagged[index] || gameOver) {
    return [];
  }
  
  // このセルを開く
  revealed[index] = true;
  
  // 開かれたセルのリスト
  const revealedCells = [index];
  
  // 地雷を開いた場合はゲームオーバー
  if (cells[index] === -1) {
    gameState.gameOver = true;
    return revealedCells;
  }
  
  // 数字が0の場合は周囲のセルも開く（再帰的に）
  if (cells[index] === 0) {
    const neighbors = getNeighbors(index);
    for (const neighbor of neighbors) {
      // 既に開かれている場合はスキップ
      if (revealed[neighbor]) continue;
      
      // 再帰的に開く
      const additionalRevealed = revealCell(neighbor);
      revealedCells.push(...additionalRevealed);
    }
  }
  
  // 勝利条件をチェック
  checkWinCondition();
  
  return revealedCells;
}

// 勝利条件をチェック
function checkWinCondition() {
  const { boardWidth, boardHeight, cells, revealed, gameOver } = gameState;
  
  if (gameOver) return;
  
  // すべての非地雷セルが開かれているかチェック
  for (let i = 0; i < boardWidth * boardHeight; i++) {
    if (cells[i] !== -1 && !revealed[i]) {
      return; // まだ開かれていないセルがある
    }
  }
  
  // 勝利！
  gameState.win = true;
  gameState.gameOver = true;
}

// フラグを切り替え
function toggleFlag(index) {
  const { revealed, flagged, gameOver } = gameState;
  
  // 既に開かれている、またはゲームオーバーの場合は何もしない
  if (revealed[index] || gameOver) {
    return;
  }
  
  // フラグを切り替え
  flagged[index] = !flagged[index];
}

// ルートアクセス時にサーバー情報を表示
app.get('/', (req, res) => {
    res.send(`
        <h1>Minesweeper WebSocket Server</h1>
        <p>This server is running on port ${PORT}</p>
        <p>Connected clients: ${clients.size}</p>
        <p>Server status: Active</p>
        <p>WebSocket URL: ws://${req.headers.host}</p>
    `);
});

// サーバー作成
const server = http.createServer(app);

// WebSocketサーバーをポートで起動
const wss = new WebSocket.Server({ server });

// サーバーを指定ポートで起動
server.listen(PORT, () => {
    console.log(`WebSocketサーバーを起動しました (ポート: ${PORT})`);
});

// ゲームを初期化
initializeGame();

// 定期的にピングを送信して接続を維持
setInterval(() => {
    for (const client of clients.keys()) {
        if (client.readyState === WebSocket.OPEN) {
            client.ping();
        }
    }
}, 30000);

// 接続イベント
wss.on('connection', (ws) => {
  // 新しいクライアントにIDを付与
  const playerId = `player_${nextPlayerId++}`;
  console.log(`新しいプレイヤーが接続しました: ${playerId}`);
  
  // クライアントをマップに保存
  clients.set(ws, { 
    id: playerId,
    x: 0,
    y: 0,
    color: generateRandomColor()
  });
  
  // 接続しているプレイヤー情報を新規クライアントに送信
  const playerList = [];
  for (const [client, data] of clients.entries()) {
    if (client !== ws) {
      playerList.push({ 
        id: data.id,
        x: data.x,
        y: data.y,
        color: data.color
      });
    }
  }
  
  // 初期化メッセージを送信
  ws.send(JSON.stringify({
    type: 'init',
    playerId: playerId,
    players: playerList,
    gameState: {
      boardWidth: gameState.boardWidth,
      boardHeight: gameState.boardHeight,
      mineCount: gameState.mineCount,
      revealed: gameState.revealed,
      flagged: gameState.flagged,
      gameStarted: gameState.gameStarted,
      gameOver: gameState.gameOver,
      win: gameState.win
    }
  }));
  
  // 他のプレイヤーに新規参加を通知
  const joinMessage = JSON.stringify({
    type: 'player_joined',
    id: playerId,
    color: clients.get(ws).color
  });
  
  broadcastExcept(ws, joinMessage);
  
  // メッセージ受信イベント
  ws.on('message', (message) => {
    try {
      const data = JSON.parse(message.toString());
      const clientInfo = clients.get(ws);
      
      switch (data.type) {
        case 'position_update':
          // プレイヤーの位置更新
          clientInfo.x = data.x;
          clientInfo.y = data.y;
          
          const updateMessage = JSON.stringify({
            type: 'player_moved',
            id: clientInfo.id,
            x: data.x,
            y: data.y
          });
          
          broadcastExcept(ws, updateMessage);
          break;
          
        case 'reveal_cell':
          // セルを開く
          const index = data.index;
          
          // ゲームが開始されていない場合は、最初のクリックで開始
          if (!gameState.gameStarted) {
            gameState.gameStarted = true;
            placeMines(index);
          }
          
          // セルを開く
          const revealedCells = revealCell(index);
          
          // 結果を全員に送信
          if (revealedCells.length > 0) {
            const cellValues = {};
            for (const idx of revealedCells) {
              cellValues[idx] = gameState.cells[idx];
            }
            
            const revealMessage = JSON.stringify({
              type: 'cells_revealed',
              cells: revealedCells,
              values: cellValues,
              gameOver: gameState.gameOver,
              win: gameState.win,
              playerId: clientInfo.id
            });
            
            broadcast(revealMessage);
          }
          break;
          
        case 'toggle_flag':
          // フラグを切り替え
          const flagIndex = data.index;
          toggleFlag(flagIndex);
          
          const flagMessage = JSON.stringify({
            type: 'flag_toggled',
            index: flagIndex,
            flagged: gameState.flagged[flagIndex],
            playerId: clientInfo.id
          });
          
          broadcast(flagMessage);
          break;
          
        case 'reset_game':
          // ゲームをリセット
          initializeGame();
          
          const resetMessage = JSON.stringify({
            type: 'game_reset',
            gameState: {
              boardWidth: gameState.boardWidth,
              boardHeight: gameState.boardHeight,
              mineCount: gameState.mineCount,
              revealed: gameState.revealed,
              flagged: gameState.flagged,
              gameStarted: gameState.gameStarted,
              gameOver: gameState.gameOver,
              win: gameState.win
            }
          });
          
          broadcast(resetMessage);
          break;
      }
    } catch (error) {
      console.error('メッセージ解析エラー:', error);
    }
  });
  
  // 切断イベント
  ws.on('close', () => {
    const clientInfo = clients.get(ws);
    if (clientInfo) {
      console.log(`プレイヤーが切断しました: ${clientInfo.id}`);
      
      // 切断したプレイヤーを全員に通知
      const leaveMessage = JSON.stringify({
        type: 'player_left',
        id: clientInfo.id
      });
      broadcastExcept(ws, leaveMessage);
      
      // クライアントマップから削除
      clients.delete(ws);
      
      // プレイヤーがいなくなったらゲームをリセット
      if (clients.size === 0) {
        initializeGame();
      }
    }
  });
});

// 全クライアントにメッセージを送信
function broadcast(message) {
  for (const client of clients.keys()) {
    if (client.readyState === WebSocket.OPEN) {
      client.send(message);
    }
  }
}

// 特定のクライアントを除いて全員にメッセージ送信
function broadcastExcept(excludeWs, message) {
  for (const client of clients.keys()) {
    if (client !== excludeWs && client.readyState === WebSocket.OPEN) {
      client.send(message);
    }
  }
}

// ランダムな色を生成（明るめの色）
function generateRandomColor() {
  const h = Math.floor(Math.random() * 360);
  const s = 70 + Math.floor(Math.random() * 30); // 70-100%
  const l = 50 + Math.floor(Math.random() * 10); // 50-60%
  return `hsl(${h}, ${s}%, ${l}%)`;
}
