import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import styles from './ConfigTab.module.css'

export function ConfigTab() {
  const [configPath, setConfigPath] = useState('')
  const [content, setContent] = useState('')
  const [msg, setMsg] = useState('')
  const [msgType, setMsgType] = useState<'ok' | 'err'>('ok')

  const load = async () => {
    try {
      const [path, text] = await Promise.all([
        invoke<string>('get_config_path'),
        invoke<string>('load_config_cmd'),
      ])
      setConfigPath(path)
      setContent(text)
      setMsg('')
    } catch (e) {
      setMsg(String(e))
      setMsgType('err')
    }
  }

  useEffect(() => { load() }, [])

  const save = async () => {
    try {
      await invoke('save_config_cmd', { content })
      setMsg('保存成功')
      setMsgType('ok')
    } catch (e) {
      setMsg(String(e))
      setMsgType('err')
    }
  }

  const init = async () => {
    try {
      await invoke('init_config_cmd')
      setMsg('配置模板已生成，请填写 API Key')
      setMsgType('ok')
      await load()
    } catch (e) {
      setMsg(String(e))
      setMsgType('err')
    }
  }

  return (
    <div className={styles.wrap}>
      <div className={styles.group}>
        <label className={styles.label}>配置文件路径</label>
        <div className={styles.pathBox}>{configPath || '加载中…'}</div>
      </div>

      <div className={styles.group}>
        <label className={styles.label}>编辑配置</label>
        <textarea
          className={styles.editor}
          value={content}
          onChange={e => setContent(e.target.value)}
          rows={22}
          spellCheck={false}
          placeholder="配置文件内容加载中…"
        />
      </div>

      <div className={styles.actions}>
        <button className={styles.btn} onClick={save}>保存</button>
        <button className={styles.btnSecondary} onClick={init}>生成配置模板</button>
        <button className={styles.btnSecondary} onClick={load}>重新加载</button>
      </div>

      {msg && (
        <div className={msgType === 'ok' ? styles.msgOk : styles.msgErr}>{msg}</div>
      )}

      <p className={styles.note}>
        注意：API Key 字段请直接编辑磁盘文件或使用 macOS Keychain 存储，此处显示文件原文。
      </p>
    </div>
  )
}
