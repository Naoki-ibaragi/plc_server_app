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
        const unlistenMessage = await listen('plc-message', (event) => {
          const { plc_id, message, timestamp } = event.payload;

          // nullや空データの場合は更新しない(切断時に無効なデータが送られてくる場合があるため)
          if (!message || message === "" || (typeof message === 'object' && Object.keys(message).length === 0)) {
            console.log(`[MESSAGE] PLC ${plc_id}: 空またはnullのデータを受信したためスキップ`);
            return;
          }

          console.log(`[MESSAGE] PLC ${plc_id} received data at ${timestamp}`);

          //対象のplc_idのplcListのlastReceivedをmessageで更新
          setPlcList((prev) => {
            const targetPlc = prev.find(p => p.id === plc_id);
            console.log(`[MESSAGE] PLC ${plc_id} current status: ${targetPlc?.status}`);

            return prev.map((p) =>
              p.id === plc_id
                ? { ...p, lastReceived: timestamp, data:message }
                : p
            );
          });
        });

        const unlistenDisconnect = await listen('plc-disconnected', (event) => {
          const { plc_id, reason } = event.payload;

          console.log('========== PLC DISCONNECT EVENT ==========');
          console.log(`[DISCONNECT] Timestamp: ${new Date().toISOString()}`);
          console.log(`[DISCONNECT] PLC ID: ${plc_id}`);
          console.log(`[DISCONNECT] Reason: ${reason}`);

          // イベント受信時の現在の状態をログ出力
          setPlcList((prev) => {
            const targetPlc = prev.find(p => p.id === plc_id);
            console.log(`[DISCONNECT] Current state BEFORE update:`, {
              id: targetPlc?.id,
              name: targetPlc?.name,
              status: targetPlc?.status,
              lastReceived: targetPlc?.lastReceived,
              hasData: !!targetPlc?.data
            });

            const updated = prev.map((p) =>
              p.id === plc_id
                ? { ...p, status: "disconnected" }
                : p
            );

            const updatedPlc = updated.find(p => p.id === plc_id);
            console.log(`[DISCONNECT] New state AFTER update:`, {
              id: updatedPlc?.id,
              name: updatedPlc?.name,
              status: updatedPlc?.status,
              lastReceived: updatedPlc?.lastReceived,
              hasData: !!updatedPlc?.data
            });
            console.log('==========================================');

            return updated;
          });
        });

        return () => {
          unlistenMessage();
          unlistenDisconnect();
        };
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
      console.log('========== PLC CONNECT REQUEST ==========');
      console.log(`[CONNECT] Timestamp: ${new Date().toISOString()}`);
      console.log(`[CONNECT] PLC ID: ${plc.id}`);
      console.log(`[CONNECT] PLC Name: ${plc.name}`);
      console.log(`[CONNECT] Current status: ${plc.status}`);

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
      });

      console.log(`[CONNECT] Backend connection successful for PLC ${plc.id}`);

      // 接続成功したらステータスを更新
      setPlcList((prev) => {
        const updated = prev.map((p) =>
          p.id === plc.id
            ? { ...p, status: "connected", lastReceived: new Date().toLocaleString("ja-JP") }
            : p
        );
        console.log(`[CONNECT] Frontend status updated to 'connected' for PLC ${plc.id}`);
        console.log('==========================================');
        return updated;
      });
    } catch (err) {
      console.error(`[CONNECT ERROR] Failed to connect to PLC ${plc.id}:`, err);
      console.log('==========================================');
      alert(`接続に失敗しました: ${err}`);
      throw err;
    }
  };

  // PLC切断処理
  const handleDisconnect = async (plc) => {
    try {
      console.log('========== PLC MANUAL DISCONNECT REQUEST ==========');
      console.log(`[MANUAL DISCONNECT] Timestamp: ${new Date().toISOString()}`);
      console.log(`[MANUAL DISCONNECT] PLC ID: ${plc.id}`);
      console.log(`[MANUAL DISCONNECT] PLC Name: ${plc.name}`);
      console.log(`[MANUAL DISCONNECT] Current status: ${plc.status}`);

      // Rust側の切断コマンドを呼び出す
      await invoke("disconnect_plc", { plcId: plc.id });

      console.log(`[MANUAL DISCONNECT] Backend disconnection successful for PLC ${plc.id}`);

      // 切断成功したらステータスを更新(最終受信データと時刻は保持)
      setPlcList((prev) => {
        const updated = prev.map((p) =>
          p.id === plc.id ? { ...p, status: "disconnected" } : p
        );
        console.log(`[MANUAL DISCONNECT] Frontend status updated to 'disconnected' for PLC ${plc.id}`);
        console.log('===================================================');
        return updated;
      });
    } catch (err) {
      console.error(`[MANUAL DISCONNECT ERROR] Failed to disconnect from PLC ${plc.id}:`, err);
      console.log('===================================================');
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
      });

      console.log("newConfig",newConfig);

      // 設定リストに追加
      //今回追加した最終要素を取り出し
      const last_item=newConfig.at(-1);
      setPlcConfigs((prev) => [...prev, last_item]);

      // 表示リストに追加
      setPlcList((prev) => [
        ...prev,
        {
          id: last_item.id,
          name: last_item.name,
          status: "disconnected",
          ip: last_item.plc_ip,
          port: last_item.plc_port,
          lastReceived: "-",
          data: null,
        },
      ]);

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
        id: formData.id,
        name: formData.name,
        plcIp: formData.plc_ip,
        plcPort: parseInt(formData.plc_port),
        pcIp: formData.pc_ip,
      });

      console.log("handleEdit new config",newConfig);

      // 設定リストを更新
      setPlcConfigs(newConfig);

      // 表示リストを更新（該当IDのPLCのみ更新し、status/lastReceived/dataは保持）
      setPlcList((prev) =>
        prev.map((p) =>
          p.id === formData.id
            ? {
                ...p,
                name: formData.name,
                ip: formData.plc_ip,
                port: parseInt(formData.plc_port),
              }
            : p
        )
      );

      alert("編集が完了しました");
    } catch (err) {
      console.error("Failed to edit PLC:", err);
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
          <h1 className="text-2xl font-bold">設備情報収集システム</h1>
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
