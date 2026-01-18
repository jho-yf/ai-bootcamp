/**
 * SqlEditor 组件
 * Monaco Editor 包装器，配置 SQL 语法高亮
 */
import React from 'react';
import Editor from '@monaco-editor/react';

interface SqlEditorProps {
  value: string;
  onChange?: (value: string | undefined) => void;
  height?: string;
  readOnly?: boolean;
}

export const SqlEditor: React.FC<SqlEditorProps> = ({
  value,
  onChange,
  height = '400px',
  readOnly = false,
}) => {
  const handleEditorDidMount = () => {
    // Editor 已挂载，可以在这里配置快捷键等
  };

  return (
    <div style={{
      height: height === '100%' ? '100%' : height,
      width: '100%',
      padding: '0 16px',
      boxSizing: 'border-box'
    }}>
      <Editor
        height={height === '100%' ? '100%' : height}
        language="sql"
        theme="vs-dark"
        value={value}
        onChange={onChange}
        onMount={handleEditorDidMount}
        options={{
          minimap: { enabled: false },
          fontSize: 14,
          lineNumbers: 'on',
          automaticLayout: true,
          readOnly,
          wordWrap: 'on',
          scrollBeyondLastLine: false,
          suggestOnTriggerCharacters: true,
          quickSuggestions: true,
          padding: { top: 16, bottom: 16 },
        }}
      />
    </div>
  );
};
