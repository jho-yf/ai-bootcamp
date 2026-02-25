// src/components/UpdateDialog.tsx

import { useEffect, useState } from "react";
import { updaterApi, updateEvents, type UpdateStatus, type UpdateInfo } from "../api/tauri";

interface UpdateDialogProps {
  updateUrl: string;
}

export function UpdateDialog({ updateUrl }: UpdateDialogProps) {
  const [status, setStatus] = useState<UpdateStatus>({ type: "UpToDate" });
  const [showDialog, setShowDialog] = useState(false);

  useEffect(() => {
    const unlisten = updateEvents.onUpdateStatusChanged((newStatus) => {
      setStatus(newStatus);
      if (newStatus.type === "Available" || newStatus.type === "ReadyToInstall") {
        setShowDialog(true);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleCheckUpdates = async () => {
    try {
      const info = await updaterApi.checkForUpdates(updateUrl);
      setStatus({
        type: "Available",
        version: info.version,
        notes: info.notes,
      });
      setShowDialog(true);
    } catch (error) {
      setStatus({ type: "Error", message: String(error) });
    }
  };

  const handleInstallUpdate = () => {
    // 使用 Tauri 内置更新器安装更新
    window.location.reload();
  };

  const handleDismiss = () => {
    setShowDialog(false);
  };

  return (
    <>
      {!showDialog && (
        <button
          onClick={handleCheckUpdates}
          className="px-4 py-2 bg-gray-700 text-white rounded hover:bg-gray-600 transition"
        >
          检查更新
        </button>
      )}

      {showDialog && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl p-6 max-w-md w-full mx-4 dark:bg-gray-800">
            {status.type === "Checking" && (
              <div className="text-center">
                <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
                <p className="text-gray-700 dark:text-gray-300">正在检查更新...</p>
              </div>
            )}

            {status.type === "Available" && (
              <>
                <h2 className="text-xl font-bold mb-4 text-gray-900 dark:text-white">
                  发现新版本 {status.version}
                </h2>
                <div className="mb-4">
                  <h3 className="font-semibold mb-2 text-gray-800 dark:text-gray-200">
                    更新说明:
                  </h3>
                  <p className="text-gray-600 dark:text-gray-400 whitespace-pre-wrap">
                    {status.notes}
                  </p>
                </div>
                <div className="flex gap-3 justify-end">
                  <button
                    onClick={handleDismiss}
                    className="px-4 py-2 bg-gray-200 text-gray-800 rounded hover:bg-gray-300 transition dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600"
                  >
                    稍后提醒
                  </button>
                  <button
                    onClick={handleInstallUpdate}
                    className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition"
                  >
                    立即更新
                  </button>
                </div>
              </>
            )}

            {status.type === "Downloading" && (
              <div className="text-center">
                <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
                <p className="text-gray-700 dark:text-gray-300">
                  正在下载更新... {status.progress}%
                </p>
              </div>
            )}

            {status.type === "ReadyToInstall" && (
              <>
                <h2 className="text-xl font-bold mb-4 text-gray-900 dark:text-white">
                  更新已准备就绪
                </h2>
                <p className="text-gray-600 dark:text-gray-400 mb-6">
                  应用需要重启以完成更新安装。
                </p>
                <div className="flex gap-3 justify-end">
                  <button
                    onClick={handleDismiss}
                    class="px-4 py-2 bg-gray-200 text-gray-800 rounded hover:bg-gray-300 transition dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600"
                  >
                    稍后重启
                  </button>
                  <button
                    onClick={handleInstallUpdate}
                    className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition"
                  >
                    立即重启
                  </button>
                </div>
              </>
            )}

            {status.type === "UpToDate" && (
              <>
                <h2 className="text-xl font-bold mb-4 text-gray-900 dark:text-white">
                  已是最新版本
                </h2>
                <p className="text-gray-600 dark:text-gray-400 mb-6">
                  您当前使用的是最新版本。
                </p>
                <div className="flex justify-end">
                  <button
                    onClick={handleDismiss}
                    className="px-4 py-2 bg-gray-200 text-gray-800 rounded hover:bg-gray-300 transition dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600"
                  >
                    关闭
                  </button>
                </div>
              </>
            )}

            {status.type === "Error" && (
              <>
                <h2 className="text-xl font-bold mb-4 text-gray-900 dark:text-white">
                  更新检查失败
                </h2>
                <p className="text-red-600 dark:text-red-400 mb-6">
                  {status.message}
                </p>
                <div className="flex gap-3 justify-end">
                  <button
                    onClick={handleDismiss}
                    className="px-4 py-2 bg-gray-200 text-gray-800 rounded hover:bg-gray-300 transition dark:bg-gray-700 dark:text-gray-200 dark:hover:bg-gray-600"
                  >
                    关闭
                  </button>
                  <button
                    onClick={handleCheckUpdates}
                    className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition"
                  >
                    重试
                  </button>
                </div>
              </>
            )}
          </div>
        </div>
      )}
    </>
  );
}
