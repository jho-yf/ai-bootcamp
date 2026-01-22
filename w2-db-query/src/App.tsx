import { ConfigProvider } from "antd";
import zhCN from "antd/locale/zh_CN";
import { AppLayout } from "./components/Layout";
import "./App.css";

function App() {
  return (
    <ConfigProvider locale={zhCN}>
      <AppLayout />
    </ConfigProvider>
  );
}

export default App;
