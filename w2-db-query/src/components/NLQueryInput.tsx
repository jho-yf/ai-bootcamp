/**
 * NLQueryInput 组件
 * 自然语言查询输入框和生成按钮
 */
import React, { useState } from "react";
import { Input } from "antd";

const { TextArea } = Input;

interface NLQueryInputProps {
  /** 数据库 ID */
  databaseId?: string | null;
  /** 是否正在加载 */
  loading?: boolean;
  /** 生成的 SQL（用于显示） */
  generatedSql?: string;
  /** 自然语言查询输入文本（受控组件） */
  prompt?: string;
  /** 自然语言查询输入变化回调 */
  onPromptChange?: (prompt: string) => void;
  /** 生成 SQL 回调（仅生成，不执行） */
  onGenerate?: (prompt: string) => Promise<string>;
  /** 执行查询回调（生成并执行） */
  onExecute?: (prompt: string) => Promise<void>;
  /** 清空生成的 SQL 回调 */
  onClear?: () => void;
}

export const NLQueryInput: React.FC<NLQueryInputProps> = ({
  databaseId: _databaseId,
  loading = false,
  generatedSql,
  prompt: controlledPrompt,
  onPromptChange,
  onGenerate: _onGenerate,
  onExecute,
  onClear: _onClear,
}) => {
  const [internalPrompt, setInternalPrompt] = useState("");
  const prompt =
    controlledPrompt !== undefined ? controlledPrompt : internalPrompt;
  const setPrompt = onPromptChange || setInternalPrompt;

  // 处理 Ctrl+Enter 快捷键
  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      if (onExecute && prompt.trim()) {
        onExecute(prompt.trim());
      }
    }
  };

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        minHeight: 0,
      }}
    >
      {/* 输入框 */}
      <div
        style={{
          flex: 1,
          marginBottom: 12,
          minHeight: 0,
          display: "flex",
          flexDirection: "column",
        }}
      >
        <TextArea
          placeholder="使用自然语言描述您的查询需求，例如：查询所有年龄大于30岁的用户姓名和邮箱"
          value={prompt}
          onChange={(e) => setPrompt(e.target.value)}
          disabled={loading}
          rows={10}
          style={{ resize: "none", fontFamily: "monospace", flex: 1 }}
          onKeyDown={handleKeyDown}
        />
      </div>

      {generatedSql && (
        <div
          style={{
            marginTop: 12,
            padding: 12,
            background: "#f5f5f5",
            borderRadius: 4,
          }}
        >
          <div style={{ fontSize: 12, color: "#666", marginBottom: 8 }}>
            生成的 SQL：
          </div>
          <pre
            style={{
              margin: 0,
              fontSize: 12,
              fontFamily: "monospace",
              whiteSpace: "pre-wrap",
              wordBreak: "break-all",
              maxHeight: 150,
              overflow: "auto",
            }}
          >
            {generatedSql}
          </pre>
        </div>
      )}
    </div>
  );
};
