import { BrowserRouter, Routes, Route } from "react-router-dom";
import { TicketsPage } from "@/pages/TicketsPage";

/**
 * 主应用组件
 * Phase 3: 页面开发
 * - 配置 React Router
 * - 定义路由结构
 */
function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<TicketsPage />} />
      </Routes>
    </BrowserRouter>
  );
}

export default App;
