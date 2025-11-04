import React, { useState } from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { ChevronDown, X, CheckCircle, XCircle, Wifi } from "lucide-react";

// サンプルデータ
const samplePlcData = [
  {
    id: 1,
    name: "PLC-1 (成型機A)",
    status: "connected",
    ip: "192.168.1.100",
    port: 5000,
    lastReceived: "2024-11-04 14:30:25",
    data: {
      temperature: 85.5,
      pressure: 120,
      cycleCount: 1234,
    },
  },
  {
    id: 2,
    name: "PLC-2 (成型機B)",
    status: "connected",
    ip: "192.168.1.101",
    port: 5000,
    lastReceived: "2024-11-04 14:30:23",
    data: {
      temperature: 88.2,
      pressure: 118,
      cycleCount: 987,
    },
  },
  {
    id: 3,
    name: "PLC-3 (成型機C)",
    status: "disconnected",
    ip: "192.168.1.102",
    port: 5000,
    lastReceived: "2024-11-04 14:25:10",
    data: null,
  },
  {
    id: 4,
    name: "PLC-4 (搬送ライン)",
    status: "connected",
    ip: "192.168.1.103",
    port: 5001,
    lastReceived: "2024-11-04 14:30:24",
    data: {
      speed: 45,
      position: 1250,
      itemCount: 450,
    },
  },
];

function PLCCard({ plc }) {
  const [isExpanded, setIsExpanded] = useState(false);
  const isConnected = plc.status === "connected";

  return (
    <div className="bg-gray-800 rounded-lg mb-2 overflow-hidden shadow-md">
      {/* カードヘッダー（常に表示） */}
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full p-4 flex items-center gap-3 hover:bg-gray-750 transition-colors"
      >
        <Wifi className={isConnected ? "text-blue-400" : "text-gray-500"} size={24} />
        <div className="flex-1 text-left">
          <h3 className="text-lg font-semibold text-white">{plc.name}</h3>
        </div>
        <span
          className={`flex items-center gap-1 px-3 py-1 rounded-full text-sm font-medium ${
            isConnected
              ? "bg-green-900/50 text-green-400"
              : "bg-red-900/50 text-red-400"
          }`}
        >
          {isConnected ? (
            <>
              <CheckCircle size={16} />
              接続中
            </>
          ) : (
            <>
              <XCircle size={16} />
              切断
            </>
          )}
        </span>
        <ChevronDown
          className={`text-gray-400 transition-transform ${
            isExpanded ? "rotate-180" : ""
          }`}
          size={20}
        />
      </button>

      {/* 展開時の詳細情報 */}
      {isExpanded && (
        <div className="px-4 pb-4 border-t border-gray-700">
          <div className="pt-4 space-y-4">
            <div>
              <p className="text-sm text-gray-400 mb-1">IPアドレス</p>
              <p className="text-white font-mono">{plc.ip}</p>
            </div>

            <div>
              <p className="text-sm text-gray-400 mb-1">ポート</p>
              <p className="text-white font-mono">{plc.port}</p>
            </div>

            <div>
              <p className="text-sm text-gray-400 mb-1">最終受信時刻</p>
              <p className="text-white">{plc.lastReceived}</p>
            </div>

            <div className="border-t border-gray-700 pt-4">
              <p className="text-sm text-gray-400 mb-2">受信データ</p>
              {plc.data ? (
                <div className="bg-gray-900 p-3 rounded border border-gray-700">
                  <pre className="text-sm text-green-400 font-mono overflow-x-auto">
                    {JSON.stringify(plc.data, null, 2)}
                  </pre>
                </div>
              ) : (
                <p className="text-gray-500 italic">データなし</p>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default function StackCard() {
  const [plcList] = useState(samplePlcData);

  const hideWindow = async () => {
    try {
      const appWindow = getCurrentWindow();
      await appWindow.hide();
    } catch (error) {
      console.error("Failed to hide window:", error);
    }
  };

  const connectedCount = plcList.filter((plc) => plc.status === "connected").length;

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      {/* ヘッダー */}
      <header className="bg-gray-800 shadow-lg">
        <div className="flex items-center justify-between p-4">
          <h1 className="text-2xl font-bold">PLC監視システム</h1>
          <div className="flex items-center gap-4">
            <span className="px-3 py-1 bg-blue-900/50 text-blue-400 rounded-full text-sm font-medium">
              {connectedCount}/{plcList.length} 接続中
            </span>
            <button
              onClick={hideWindow}
              className="p-2 hover:bg-gray-700 rounded-full transition-colors"
              aria-label="ウィンドウを閉じる"
            >
              <X size={24} />
            </button>
          </div>
        </div>
      </header>

      {/* メインコンテンツ */}
      <main className="p-6">
        <h2 className="text-xl font-semibold mb-4">接続中のPLC</h2>
        <div className="space-y-2">
          {plcList.map((plc) => (
            <PLCCard key={plc.id} plc={plc} />
          ))}
        </div>
      </main>
    </div>
  );
}
