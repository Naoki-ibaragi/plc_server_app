import React, { useState } from "react";
import { ChevronDown, CheckCircle, XCircle, Wifi, Play, Square, Cog, Trash2 } from "lucide-react";
import EditPlcDialog from "./EditPlcDialog";

/**
 * 個別のPLCカードコンポーネント
 * @param {Object} props
 * @param {Object} props.plc - PLC情報
 * @param {Object} props.config - ソケット情報
 * @param {Function} props.onConnect - 接続ハンドラー
 * @param {Function} props.onDisconnect - 切断ハンドラー
 * @param {Function} props.onDelete - 削除ハンドラー
 * @param {Function} props.onEdit - 編集ハンドラー
 */
export default function PLCCard({ plc, config,onConnect, onDisconnect, onDelete, onEdit }) {
  const [isExpanded, setIsExpanded] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);
  const isConnected = plc.status === "connected";

  const handleConnect = async (e) => {
    e.stopPropagation();
    setIsConnecting(true);
    try {
      await onConnect(plc);
    } catch (error) {
      console.error("Connection failed:", error);
    } finally {
      setIsConnecting(false);
    }
  };

  const handleDisconnect = async (e) => {
    e.stopPropagation();
    try {
      await onDisconnect(plc);
    } catch (error) {
      console.error("Disconnection failed:", error);
    }
  };

  const handleEdit = async (e) => {
    e.stopPropagation();
    if (isConnected) {
      alert("接続中のPLC情報は編集できません。先に切断してください。");
      return;
    }
    //編集用のダイアログを表示
    setIsEditDialogOpen(true);
  };

  const handleDelete = async (e) => {
    e.stopPropagation();
    if (isConnected) {
      alert("接続中のPLCは削除できません。先に切断してください。");
      return;
    }
    if (confirm(`${plc.name}を削除してもよろしいですか？`)) {
      try {
        await onDelete(plc);
      } catch (error) {
        console.error("Deletion failed:", error);
      }
    }
  };

  return (
    <div className={isExpanded ? "bg-gray-700 rounded-lg mb-2 overflow-hidden shadow-md":"bg-gray-800 rounded-lg mb-2 overflow-hidden shadow-md"}>
      {/* カードヘッダー（常に表示） */}
      <div className="w-full p-4 flex items-center gap-3">
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className="flex-1 flex items-center gap-3 hover:bg-gray-750 transition-colors rounded p-2 -m-2"
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

        {/* 接続/切断ボタン */}
        {isConnected ? (
          <button
            onClick={handleDisconnect}
            className="flex items-center gap-2 px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg transition-colors"
          >
            <Square size={16} />
            切断
          </button>
        ) : (
          <button
            onClick={handleConnect}
            disabled={isConnecting}
            className={`flex items-center gap-2 px-4 py-2 bg-blue-700 hover:bg-blue-600 text-white rounded-lg transition-colors ${
              isConnecting ? "opacity-50 cursor-not-allowed" : ""
            }`}
          >
            <Play size={16} />
            {isConnecting ? "接続中..." : "接続"}
          </button>
        )}
      </div>

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
              <p className="text-sm text-gray-400 mb-1">テーブル名</p>
              <p className="text-white font-mono">{plc.table_name}</p>
            </div>

            <div>
              <p className="text-sm text-gray-400 mb-1">最終受信時刻</p>
              <p className="text-white">{plc.lastReceived}</p>
            </div>

            <div className="border-t border-gray-700 pt-4">
              <p className="text-sm text-gray-400 mb-2">受信データ</p>
              {plc.data ? (
                <div className="bg-gray-900 p-3 rounded border border-gray-700">
                  <pre className="text-sm text-green-400 font-mono overflow-y-auto max-h-20 whitespace-pre-wrap break-all">
                    {JSON.stringify(plc.data, null, 2)}
                  </pre>
                </div>
              ) : (
                <p className="text-gray-500 italic">データなし</p>
              )}
            </div>
            {/* 情報編集ボタン */}
            <div className="border-t border-gray-700 pt-4">
              <button
                onClick={handleEdit}
                disabled={isConnected}
                className={`w-full flex items-center justify-center gap-2 px-4 py-1 rounded-lg transition-colors ${
                  isConnected
                    ? "bg-gray-700 text-gray-500 cursor-not-allowed"
                    : "bg-green-700 hover:bg-green-600 text-white"
                }`}
              >
                <Cog size={16} />
                {isConnected ? "接続中は編集できません" : "このPLC情報を編集"}
              </button>
            </div>

            {/* 削除ボタン */}
            <div className="border-t border-gray-700 pt-4">
              <button
                onClick={handleDelete}
                disabled={isConnected}
                className={`w-full flex items-center justify-center gap-2 px-4 py-1 rounded-lg transition-colors ${
                  isConnected
                    ? "bg-gray-700 text-gray-500 cursor-not-allowed"
                    : "bg-red-700 hover:bg-red-600 text-white"
                }`}
              >
                <Trash2 size={16} />
                {isConnected ? "接続中は削除できません" : "このPLCを削除"}
              </button>
            </div>
          </div>
        </div>
      )}
      {/* PLC情報編集ダイアログ */}
      <EditPlcDialog
        plc={plc}
        config={config}
        isOpen={isEditDialogOpen}
        onClose={() => setIsEditDialogOpen(false)}
        onEdit={onEdit}
      />
    </div>
  );
}
