# MirrorSphere

一個用 Rust 開發的高效能、跨平台圖形化檔案備份工具。

## 特色

- 用戶友好的圖形界面
- 高效能平行備份演算法
- 跨平台支援 (Windows, Linux)
- 增量備份能力
- 檔案系統變更追蹤 (Windows NTFS)
- 支援 Windows NTFS ACL
- 保留額外檔案屬性
- 實時備份進度顯示

## 安裝

### 下載安裝程式

請查看 [Releases](https://github.com/DaLaw2/mirrorsphere/releases) 頁面下載最新版本的安裝程式。

### 從原始碼編譯

```bash
git clone https://github.com/DaLaw2/mirrorsphere.git
cd mirrorsphere
cargo build --release
```

## 使用方法

1. 啟動 MirrorSphere 應用程式
2. 在主界面中，點擊「新建備份任務」
3. 選擇源目錄和目標目錄
4. 設定備份選項（線程數量、是否啟用增量備份等）
5. 點擊「開始備份」按鈕開始備份過程

## 支援平台

- **Windows**: 完整支援，包括 NTFS 特定功能
- **Linux**: 完整支援基本備份功能

## 技術細節

### 平行備份演算法

MirrorSphere 使用高效能的平行處理演算法，通過並行遍歷目錄結構和檔案複製來最大化效能，特別適用於大型檔案系統和多核心處理器環境。

### Windows 特定功能

- **USN Journal 追蹤**: 利用 NTFS USN 日誌快速識別檔案系統變更
- **ACL 支援**: 保留和還原 Windows NTFS 訪問控制清單
- **擴展檔案屬性**: 保留檔案的額外元數據

## 路線圖

- [ ] 實現檔案壓縮
- [ ] 加密備份選項
- [ ] 遠端備份 (SFTP, S3 等)
- [ ] 備份排程
- [ ] macOS 支援
