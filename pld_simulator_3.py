#送信伝文txtから読込
import socket
import threading
import time

# サーバー設定
HOST = '127.0.0.1'  # すべてのインターフェースで接続待ち
PORT=20001  # クライアントと一致させる
FILE_NAME="./clt_2_received_data.txt"
INTERVAL = 0.5  # sec


# txtファイルを読み込んで一定間隔でデータを送り続ける
def send_data(conn):
    try:
        with open(FILE_NAME,"r") as f:
            all_lines=f.read()
            each_datas=all_lines.split("\n")
            
        for data in each_datas:
            if data.strip():  # 空行をスキップ
                encoded_data = data.encode('utf-8')
                conn.sendall(encoded_data)
                print(f"[送信] {len(encoded_data)} byte")
                time.sleep(INTERVAL)
    except Exception as e:
        print(f"[エラー] 送信中に問題が発生しました: {e}")
    finally:
        conn.close()
        print(f"[切断] クライアント接続を閉じました")

# 各ポートでサーバーを起動
def start_server(host_ip, host_port):
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as server_sock:
        server_sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        server_sock.bind((host_ip, host_port))
        server_sock.listen()
        print(f"[待機中] {host_ip}:{host_port} で接続待機中...")

        while True:
            conn, addr = server_sock.accept()
            print(f"[接続] クライアント: {addr}")
            thread = threading.Thread(target=send_data, args=(conn,), daemon=True)
            thread.start()  # ✅ 修正: join()は不要。接続ごとに独立スレッドで並行処理


if __name__ == "__main__":
    thread = threading.Thread(target=start_server, args=(HOST, PORT), daemon=True)
    thread.start()

    print("[起動完了] 全サーバースレッドが稼働中です。Ctrl+Cで終了。")

    # メインスレッドを維持
    thread.join()

