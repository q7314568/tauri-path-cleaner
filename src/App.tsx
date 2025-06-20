import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import Button from '@mui/material/Button';
import Snackbar from '@mui/material/Snackbar';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import ListItemText from '@mui/material/ListItemText';
import LinearProgress from '@mui/material/LinearProgress';
import React, { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';

const App: React.FC = () => {
  const [snackbarOpen, setSnackbarOpen] = useState(false);
  const [snackbarMessage, setSnackbarMessage] = useState('');
  const [selectedPaths, setSelectedPaths] = useState<string[]>([]);
  const [deletionProgress, setDeletionProgress] = useState<number>(0);
  const [isDeleting, setIsDeleting] = useState(false);

  const selectFiles = async () => {
    try {
      const paths = await open({
        directory: false,
        multiple: true
      });
      if (paths && Array.isArray(paths)) {
        console.log('選擇的檔案路徑:', paths);
        setSelectedPaths(paths);
        setSnackbarMessage('檔案選擇成功');
        setSnackbarOpen(true);
      }
    } catch (error) {
      console.error('無法選擇檔案:', error);
      setSnackbarMessage('檔案選擇失敗');
      setSnackbarOpen(true);
    }
  };

  const selectDirectories = async () => {
    try {
      const paths = await open({
        directory: true,
        multiple: true
      });
      if (paths && Array.isArray(paths)) {
        console.log('選擇的資料夾路徑:', paths);
        setSelectedPaths(paths);

        let totalFiles = 0;
        for (const path of paths) {
          try {
            const count: number = await invoke('count_files_in_directory', { directoryPath: path });
            totalFiles += count;
          } catch (error) {
            console.error(`無法計算目錄內的文件數量: ${path}`, error);
          }
        }
        console.log(totalFiles);
        
        setSnackbarMessage('資料夾選擇成功');
        setSnackbarOpen(true);
      }
    } catch (error) {
      console.error('無法選擇資料夾:', error);
      setSnackbarMessage('資料夾選擇失敗');
      setSnackbarOpen(true);
    }
  };

  const deleteSelectedPaths = async () => {
    try {
      setIsDeleting(true);
      console.log('準備刪除路徑:', selectedPaths);
      await invoke('delete_paths_with_progress', { paths: selectedPaths });
      setSnackbarMessage('所有選擇的路徑已成功刪除');
      setSnackbarOpen(true);
      setSelectedPaths([]);
    } catch (error: any) {
      console.error('無法刪除路徑:', error);
      setSnackbarMessage(`刪除路徑失敗: ${error.message || error}`);
      setSnackbarOpen(true);
    } finally {
      setIsDeleting(false);
    }
  };

  const confirmAndDelete = async () => {
    if (selectedPaths.length > 0) {
      const confirmation = await window.confirm(`你確定要刪除這些路徑嗎？: ${selectedPaths.join(', ')}`);
      if (confirmation) {
        console.log('用戶確認刪除，開始刪除過程');
        await deleteSelectedPaths();
      }
    }
  };


  useEffect(() => {
    // 監聽來自後端的 "delete-progress" 事件
    const unlisten = listen<number>('delete-progress', (event) => {
      console.log(event.payload);
      // 直接使用後端傳來的百分比數值
      setDeletionProgress(event.payload);
    });

    // 清除監聽器
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  return (
    <div>
      <Button variant="contained" color="primary" onClick={selectFiles}>
        選擇檔案
      </Button>
      <Button variant="contained" color="primary" onClick={selectDirectories} style={{ marginLeft: '10px' }}>
        選擇資料夾
      </Button>
      {selectedPaths.length > 0 && (
        <div>
          <List>
            {selectedPaths.map((path, index) => (
              <ListItem key={index}>
                <ListItemText primary={path} />
              </ListItem>
            ))}
          </List>
          <Button variant="contained" color="secondary" onClick={confirmAndDelete} disabled={isDeleting}>
            刪除選擇的檔案或資料夾
          </Button>
          {isDeleting && <LinearProgress variant="determinate" value={deletionProgress} />}
        </div>
      )}
      <Snackbar
        open={snackbarOpen}
        autoHideDuration={6000}
        onClose={() => setSnackbarOpen(false)}
        message={snackbarMessage}
      />
    </div>
  );
};

export default App;
