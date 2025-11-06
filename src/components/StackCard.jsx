import React, { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { X, Plus } from "lucide-react";
import PLCCard from "./PLCCard";
import AddPlcDialog from "./AddPlcDialog";
import { listen } from '@tauri-apps/api/event';

export default function StackCard() {
  const [plcList, setPlcList] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);
  const [plcConfigs, setPlcConfigs] = useState([]); // 元の設定データを保持
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);

  // アプリ起動時にPLC設定を読み込む
  useEffect(() => {
    const loadPlcConfig = async () => {
      try {
        const configs = await invoke("init_socket");
        setPlcConfigs(configs);
        const formattedData = configs.map((config) => ({
          id: config.id,
          name: config.name,
          status: "disconnected",
          ip: config.plc_ip,
          port: config.plc_port,
          lastReceived: "-",
          data: null,
        }));
        setPlcList(formattedData);
        setLoading(false);
      } catch (err) {
        console.error("Failed to load PLC config:", err);
        setError(err);
        setLoading(false);
      }
    };

    // listenハンドラの立上
    const openListener = async () => {
      try {
        const unlisten = await listen('plc-message', (event) => {
          const { plc_id, message, timestamp } = event.payload;
          //対象のplc_idのplcListのlastReceivedをmessageで更新
          setPlcList((prev) =>
            prev.map((p) =>
              p.id === plc_id
                ? { ...p, lastReceived: timestamp, data:message }
                : p
            )
          );
        });
        return unlisten; // unlistenを返す
      } catch (err) {
        console.error("Failed to setup listener:", err);
        setError(err);
        setLoading(false);
        return null;
      }
    }

    loadPlcConfig();
    
    // クリーンアップ関数を返す
    let unlistenFn;
    openListener().then(fn => {
      unlistenFn = fn;
    });

    return () => {
      // コンポーネントのアンマウント時にリスナーを解除
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, []);

  const handleConnect = async (plc) => {
    try {
      // 元の設定データから該当のPLC設定を取得
      const config = plcConfigs.find((c) => c.id === plc.id);
      if (!config) {
        throw new Error("PLC configuration not found");
      }

      // Rust側の接続コマンドを呼び出す
      await invoke("connect_plc", {
        plcId: plc.id,
        plcIp: config.plc_ip,
        plcPort: config.plc_port,
        pcIp: config.pc_ip,
        pcPort: config.pc_port,
      });

      // 接続成功したらステータスを更新
      setPlcList((prev) =>
        prev.map((p) =>
          p.id === plc.id
            ? { ...p, status: "connected", lastReceived: new Date().toLocaleString("ja-JP") }
            : p
        )
      );
    } catch (err) {
      console.error("Failed to connect to PLC:", err);
      alert(`接続に失敗しました: ${err}`);
      throw err;
    }
  };

  // PLC切断処理
  const handleDisconnect = async (plc) => {
    try {
      // Rust側の切断コマンドを呼び出す
      await invoke("disconnect_plc", { plcId: plc.id });

      // 切断成功したらステータスを更新
      setPlcList((prev) =>
        prev.map((p) =>
          p.id === plc.id ? { ...p, status: "disconnected", data: null } : p
        )
      );
    } catch (err) {
      console.error("Failed to disconnect from PLC:", err);
      alert(`切断に失敗しました: ${err}`);
      throw err;
    }
  };

  // PLC追加処理
  const handleAddPlc = async (formData) => {
    try {
      // Rust側のPLC追加コマンドを呼び出す
      const newConfig = await invoke("add_plc", {
        name: formData.name,
        plcIp: formData.plc_ip,
        plcPort: parseInt(formData.plc_port),
        pcIp: formData.pc_ip,
        pcPort: parseInt(formData.pc_port),
      });

      // 設定リストに追加
      setPlcConfigs((prev) => [...prev, newConfig]);

      // 表示リストに追加
      setPlcList((prev) => [
        ...prev,
        {
          id: newConfig.id,
          name: newConfig.name,
          status: "disconnected",
          ip: newConfig.plc_ip,
          port: newConfig.plc_port,
          lastReceived: "-",
          data: null,
        },
      ]);

      alert("PLCを追加しました");
    } catch (err) {
      console.error("Failed to add PLC:", err);
      alert(`PLC追加に失敗しました: ${err}`);
      throw err;
    }
  };

  // PLC情報編集処理
  const handleEditPlc = async (formData) => {
    try {
      // Rust側のPLC情報編集コマンドを呼び出す
      const newConfig = await invoke("edit_plc", {
        name: formData.name,
        plcIp: formData.plc_ip,
        plcPort: parseInt(formData.plc_port),
        pcIp: formData.pc_ip,
        pcPort: parseInt(formData.pc_port),
      });

      // 設定リストを更新
      setPlcConfigs(newConfig);

      // 表示リストを更新
      setPlcList(newConfig);
      setPlcList((prev) => prev.filter((p) =>{
        if (p.id=formData.id){
          p.name=formData.name;
          p.plcIp=formData.plc_ip;
          p.plcPort=formData.plc_port;
          p.pcIp=formData.pc_ip;
          p.pcPort=formData.pc_port;
        }
      }));

      alert("編集が完了しました");
    } catch (err) {
      console.error("Failed to delete PLC:", err);
      alert(`編集が失敗しました: ${err}`);
      throw err;
    }
  };

  // PLC削除処理
  const handleDeletePlc = async (plc) => {
    try {
      // Rust側のPLC削除コマンドを呼び出す
      await invoke("delete_plc", { plcId: plc.id });

      // 設定リストから削除
      setPlcConfigs((prev) => prev.filter((c) => c.id !== plc.id));

      // 表示リストから削除
      setPlcList((prev) => prev.filter((p) => p.id !== plc.id));

      alert("PLCを削除しました");
    } catch (err) {
      console.error("Failed to delete PLC:", err);
      alert(`PLC削除に失敗しました: ${err}`);
      throw err;
    }
  };

  const hideWindow = async () => {
    try {
      const appWindow = getCurrentWindow();
      await appWindow.hide();
    } catch (error) {
      console.error("Failed to hide window:", error);
    }
  };

  const connectedCount = plcList.filter((plc) => plc.status === "connected").length;

  // ローディング中の表示
  if (loading) {
    return (
      <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-16 w-16 border-b-2 border-blue-400 mx-auto mb-4"></div>
          <p className="text-gray-400">PLC設定を読み込み中...</p>
        </div>
      </div>
    );
  }

  // エラー時の表示
  if (error) {
    return (
      <div className="min-h-screen bg-gray-900 text-white">
        <header className="bg-gray-800 shadow-lg">
          <div className="flex items-center justify-between p-4">
            <h1 className="text-2xl font-bold">PLC監視システム</h1>
            <button
              onClick={hideWindow}
              className="p-2 hover:bg-gray-700 rounded-full transition-colors"
              aria-label="ウィンドウを閉じる"
            >
              <X size={24} />
            </button>
          </div>
        </header>
        <main className="p-6">
          <div className="bg-red-900/50 border border-red-500 rounded-lg p-4">
            <h2 className="text-xl font-semibold text-red-400 mb-2">設定ファイルの読み込みに失敗しました</h2>
            <p className="text-red-300">{error.toString()}</p>
          </div>
        </main>
      </div>
    );
  }

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
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-semibold">登録されているPLC</h2>
          <button
            onClick={() => setIsAddDialogOpen(true)}
            className="flex items-center gap-2 px-4 py-2 bg-green-700 hover:bg-green-600 text-white rounded-lg transition-colors"
          >
            <Plus size={20} />
            PLC追加
          </button>
        </div>

        <div className="space-y-2">
          {plcList.length > 0 ? (
            plcList.map((plc,index) => (
              <PLCCard
                key={plc.id}
                plc={plc}
                config={plcConfigs[index]}
                onConnect={handleConnect}
                onDisconnect={handleDisconnect}
                onDelete={handleDeletePlc}
                onEdit={handleEditPlc}
              />
            ))
          ) : (
            <p className="text-gray-400 text-center py-8">PLCが登録されていません</p>
          )}
        </div>
      </main>

      {/* PLC追加ダイアログ */}
      <AddPlcDialog
        isOpen={isAddDialogOpen}
        onClose={() => setIsAddDialogOpen(false)}
        onAdd={handleAddPlc}
      />
    </div>
  );
}
