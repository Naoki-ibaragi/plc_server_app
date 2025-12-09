import { listen } from '@tauri-apps/api/event';

// PLCエラーを受信
const unlistenError = await listen('plc-error', (event) => {
  const { plc_id, error, hex_data, timestamp } = event.payload;
  console.error(`PLC ${plc_id} error at ${timestamp}: ${error}`);
  console.error(`Hex data: ${hex_data}`);
  
  // エラーをUIに表示
  displayError(plc_id, error, hex_data);
});

// 表示関数の例
function displayMessage(plcId, message, timestamp) {
  const messageList = document.getElementById('message-list');
  const messageElement = document.createElement('div');
  messageElement.innerHTML = `
    <div class="message">
      <span class="plc-id">PLC ${plcId}</span>
      <span class="timestamp">${new Date(timestamp).toLocaleString()}</span>
      <span class="content">${message}</span>
    </div>
  `;
  messageList.appendChild(messageElement);
}