const WebSocket = require('ws');
const http = require('http');
const https = require('https'); // HTTPSモジュールを追加
const fs = require('fs'); // ファイル読み込み用
const express = require('express');
const app = express();

// サーバーの設定
// Replitでは環境変数PORTが設定されているため使用
// ローカル環境では8080をデフォルトポートとして使用
const PORT = process.env.PORT || 8080;
const HTTPS_PORT = 8443; // HTTPSポート

// SSL証明書の読み込み（ファイルが存在する場合のみ）
let httpsServer;
try {
  const credentials = {
    key: fs.readFileSync('server.key'),
    cert: fs.readFileSync('server.crt')
  };
  httpsServer = https.createServer(credentials, app);
  console.log('SSL証明書を読み込みました。HTTPSサーバーを有効化します。');
} catch (err) {
  console.log('SSL証明書が見つかりません。HTTPサーバーのみで動作します。');
  console.log('詳細エラー:', err.message);
}

// CORS対策を追加
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

// HTTPサーバー作成
const server = http.createServer(app);

// WebSocketサーバーをHTTPサーバーに接続
const wss = new WebSocket.Server({ server });

// HTTPSサーバーが存在する場合は、そこにもWebSocketサーバーを接続
let wssSecure;
if (httpsServer) {
  wssSecure = new WebSocket.Server({ server: httpsServer });

  // セキュアなWebSocketサーバーにも同じイベントハンドラを設定
  wssSecure.on('connection', handleConnection);

  // HTTPSサーバーを起動
  httpsServer.listen(HTTPS_PORT, () => {
    console.log(`セキュアなWebSocketサーバーを起動しました (ポート: ${HTTPS_PORT})`);
  });
}

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

// 接続イベントハンドラーを関数として抽出
function handleConnection(ws) {
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

  // 既に開かれているセルの値を収集
  const cellValues = {};
  for (let i = 0; i < gameState.cells.length; i++) {
    if (gameState.revealed[i]) {
      cellValues[i] = gameState.cells[i];
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
      win: gameState.win,
      cellValues: cellValues
    }
  }));

  console.log(`プレイヤー ${playerId} に初期化データを送信しました。開かれたセル数: ${Object.keys(cellValues).length}`);

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
      const data = JSON.parse(message);

      // メッセージのタイプによって処理を分ける
      switch (data.type) {
        case 'player_move':
          // プレイヤーの移動
          if (data.x !== undefined && data.y !== undefined) {
            const playerData = clients.get(ws);
            playerData.x = data.x;
            playerData.y = data.y;

            // 他のクライアントに移動を通知
            for (const client of clients.keys()) {
              if (client !== ws && client.readyState === WebSocket.OPEN) {
                client.send(JSON.stringify({
                  type: 'player_moved',
                  id: playerData.id,
                  x: data.x,
                  y: data.y
                }));
              }
            }
          }
          break;

        case 'reveal_cell':
          // セルを開く
          if (data.index !== undefined) {
            const index = data.index;

            // ゲームが開始されていない場合は、最初のクリックで開始
            if (!gameState.gameStarted) {
              gameState.gameStarted = true;
              placeMines(index);
            }

            // セルを開く
            const revealedCells = revealCell(index);

            // 開かれたセルの値をマップ
            const cellValues = {};
            for (const cellIndex of revealedCells) {
              cellValues[cellIndex] = gameState.cells[cellIndex];
            }

            // すべてのクライアントに通知
            const updateMessage = JSON.stringify({
              type: 'cells_revealed',
              cells: revealedCells,
              values: cellValues
            });

            for (const client of clients.keys()) {
              if (client.readyState === WebSocket.OPEN) {
                client.send(updateMessage);
              }
            }

            // ゲームオーバーの場合は通知
            if (gameState.gameOver) {
              // ゲームオーバー時は全てのセル情報を送信
              const allCellValues = {};
              for (let i = 0; i < gameState.cells.length; i++) {
                allCellValues[i] = gameState.cells[i];
              }

              const gameOverMessage = JSON.stringify({
                type: 'game_over',
                win: gameState.win,
                cells: gameState.cells,
                allCellValues: allCellValues
              });

              for (const client of clients.keys()) {
                if (client.readyState === WebSocket.OPEN) {
                  client.send(gameOverMessage);
                }
              }
            }
          }
          break;

        case 'toggle_flag':
          // フラグを切り替え
          if (data.index !== undefined) {
            const index = data.index;
            toggleFlag(index);

            // すべてのクライアントに通知
            const flagMessage = JSON.stringify({
              type: 'flag_toggled',
              index: index,
              flagged: gameState.flagged[index]
            });

            for (const client of clients.keys()) {
              if (client.readyState === WebSocket.OPEN) {
                client.send(flagMessage);
              }
            }
          }
          break;

        case 'reset_game':
          // ゲームをリセット
          initializeGame();

          // すべてのクライアントに通知
          const resetMessage = JSON.stringify({
            type: 'game_reset',
            boardWidth: gameState.boardWidth,
            boardHeight: gameState.boardHeight,
            mineCount: gameState.mineCount
          });

          for (const client of clients.keys()) {
            if (client.readyState === WebSocket.OPEN) {
              client.send(resetMessage);
            }
          }
          break;
      }
    } catch (error) {
      console.error('メッセージ処理エラー:', error);
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
}

// 通常の接続イベント処理を関数に置き換え
wss.on('connection', handleConnection);

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
