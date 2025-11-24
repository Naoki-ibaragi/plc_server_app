import React, { useState } from "react";
import { X } from "lucide-react";

/**
 * PLC追加ダイアログコンポーネント
 * @param {Object} props
 * @param {boolean} props.isOpen - ダイアログの開閉状態
 * @param {Function} props.onClose - 閉じるハンドラー
 * @param {Function} props.onAdd - 追加ハンドラー
 */
export default function AddPlcDialog({ isOpen, onClose, onAdd }) {
  const [formData, setFormData] = useState({
    name: "",
    table_name: "",
    plc_ip: "",
    plc_port: "",
    pc_ip: "",
  });

  const handleSubmit = async (e) => {
    e.preventDefault();
    try {
      await onAdd(formData);
      setFormData({
        name: "",
        table_name: "",
        plc_ip: "",
        plc_port: "",
        pc_ip: "",
      });
      onClose();
    } catch (error) {
      console.error("Failed to add PLC:", error);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg p-6 w-full max-w-md shadow-xl">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-bold text-white">新しいPLCを追加</h2>
          <button
            onClick={onClose}
            className="p-1 hover:bg-gray-700 rounded transition-colors"
          >
            <X size={24} className="text-gray-400" />
          </button>
        </div>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm text-gray-400 mb-1">PLC名</label>
            <input
              type="text"
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              className="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
              placeholder="例: PLC-5 (検査装置)"
              required
            />
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">DBテーブル名</label>
            <input
              type="text"
              value={formData.table_name}
              onChange={(e) => setFormData({ ...formData, table_name: e.target.value })}
              className="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
              placeholder="例: clt_table_1 (DBで使用)"
              required
            />
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">PLC IPアドレス</label>
            <input
              type="text"
              value={formData.plc_ip}
              onChange={(e) => setFormData({ ...formData, plc_ip: e.target.value })}
              className="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
              placeholder="例: 192.168.1.100"
              required
            />
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">PLCポート番号</label>
            <input
              type="number"
              value={formData.plc_port}
              onChange={(e) => setFormData({ ...formData, plc_port: e.target.value })}
              className="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
              placeholder="例: 5000"
              required
            />
          </div>

          <div>
            <label className="block text-sm text-gray-400 mb-1">PC IPアドレス</label>
            <input
              type="text"
              value={formData.pc_ip}
              onChange={(e) => setFormData({ ...formData, pc_ip: e.target.value })}
              className="w-full px-3 py-2 bg-gray-700 text-white rounded border border-gray-600 focus:border-blue-500 focus:outline-none"
              placeholder="例: 192.168.1.10"
              required
            />
          </div>

          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg transition-colors"
            >
              キャンセル
            </button>
            <button
              type="submit"
              className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg transition-colors"
            >
              追加
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
