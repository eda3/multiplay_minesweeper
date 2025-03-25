// WebSocket接続管理
let socket = null;
let isConnected = false;
let messageQueue = [];

// WebSocketに接続する関数
function connectWebSocket(url) {
  if (socket) {
    console.log("WebSocketはすでに接続されています");
    return;
  }

  console.log(`WebSocketに接続中: ${url}`);
  socket = new WebSocket(url);

  socket.onopen = function() {
    console.log("WebSocket接続完了!");
    isConnected = true;
  };

  socket.onmessage = function(event) {
    console.log("メッセージを受信:", event.data);
    // メッセージをキューに追加
    messageQueue.push(event.data);
  };

  socket.onerror = function(error) {
    console.error("WebSocketエラー:", error);
  };

  socket.onclose = function() {
    console.log("WebSocket接続が閉じられました");
    isConnected = false;
    socket = null;
  };
}

// メッセージを送信する関数
function sendWebSocketMessage(message) {
  if (!socket || !isConnected) {
    console.error("WebSocketに接続されていません");
    return false;
  }

  try {
    socket.send(message);
    return true;
  } catch (error) {
    console.error("メッセージ送信エラー:", error);
    return false;
  }
}

// メッセージキューから次のメッセージを取得
function getNextWebSocketMessage() {
  if (messageQueue.length > 0) {
    return messageQueue.shift();
  }
  return null;
}

// 接続状態を確認
function isWebSocketConnected() {
  return isConnected;
}

// WebSocketを閉じる
function closeWebSocket() {
  if (socket) {
    socket.close();
    isConnected = false;
    socket = null;
  }
}
