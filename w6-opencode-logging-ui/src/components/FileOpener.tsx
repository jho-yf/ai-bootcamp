import { useCallback } from 'react'

interface Props {
  onFileSelect: (content: string, fileName: string) => void
  onDirSelect: (files: Map<string, string>, dirName: string) => void
}

export default function FileOpener({ onFileSelect, onDirSelect }: Props) {
  const handleFile = useCallback(
    (file: File) => {
      const reader = new FileReader()
      reader.onload = () => {
        onFileSelect(reader.result as string, file.name)
      }
      reader.readAsText(file)
    },
    [onFileSelect],
  )

  const handleInputChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const files = e.target.files
      if (!files || files.length === 0) return
      handleFile(files[0]!)
    },
    [handleFile],
  )

  const handleDirInputChange = useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const files = e.target.files
      if (!files || files.length === 0) return

      const fileMap = new Map<string, string>()
      for (const file of Array.from(files)) {
        const path = file.webkitRelativePath
        if (path.endsWith('.jsonl')) {
          const text = await file.text()
          fileMap.set(path, text)
        }
      }
      onDirSelect(fileMap, files[0]!.webkitRelativePath.split('/')[0]!)
    },
    [onDirSelect],
  )

  return (
    <div className="file-opener-actions" style={{ display: 'flex', gap: 'var(--space-sm)', alignItems: 'center' }}>
      <button
        className="btn btn-secondary"
        onClick={() => document.getElementById('file-input')?.click()}
      >
        Open File
      </button>
      <button
        className="btn btn-secondary"
        onClick={() => document.getElementById('dir-input')?.click()}
      >
        Open Dir
      </button>
      <input
        id="file-input"
        type="file"
        accept=".jsonl"
        style={{ display: 'none' }}
        onChange={handleInputChange}
      />
      <input
        id="dir-input"
        type="file"
        accept=".jsonl"
        style={{ display: 'none' }}
        {...({ webkitdirectory: '', directory: '' } as Record<string, string>)}
        onChange={handleDirInputChange}
      />
    </div>
  )
}
