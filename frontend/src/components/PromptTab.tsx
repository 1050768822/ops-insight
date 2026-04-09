import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { PromptConfigDto } from '../types'
import styles from './PromptTab.module.css'

const PROMPT_VARIABLES = [
  '{{period}}',
  '{{total_logs}}',
  '{{warning_count}}',
  '{{error_count}}',
  '{{fatal_count}}',
  '{{events}}',
]

const PLACEHOLDER_ZH = `你是一位资深 DevOps 工程师。请结合以下变量分析日志，并只返回 JSON。

时间范围：{{period}}
日志总数：{{total_logs}}
Warning：{{warning_count}}
Error：{{error_count}}
Fatal：{{fatal_count}}

异常事件：
{{events}}`

const PLACEHOLDER_EN = `You are a senior DevOps engineer. Analyze the logs using the variables below and return JSON only.

Time Range: {{period}}
Total Logs: {{total_logs}}
Warning: {{warning_count}}
Error: {{error_count}}
Fatal: {{fatal_count}}

Events:
{{events}}`

export function PromptTab() {
  const [prompt, setPrompt] = useState<PromptConfigDto>({ zh: '', en: '' })
  const [loading, setLoading] = useState(false)
  const [msg, setMsg] = useState('')
  const [msgType, setMsgType] = useState<'ok' | 'err'>('ok')

  const load = async () => {
    setLoading(true)
    try {
      const result = await invoke<PromptConfigDto>('load_prompt_config')
      setPrompt(result)
      setMsg('')
    } catch (e) {
      setMsg(String(e))
      setMsgType('err')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { load() }, [])

  const save = async () => {
    setLoading(true)
    try {
      await invoke('save_prompt_config', { prompt })
      setMsg('Prompt 保存成功')
      setMsgType('ok')
    } catch (e) {
      setMsg(String(e))
      setMsgType('err')
    } finally {
      setLoading(false)
    }
  }

  const resetCurrent = (language: 'zh' | 'en') => {
    setPrompt(current => ({
      ...current,
      [language]: '',
    }))
  }

  return (
    <div className={styles.wrap}>
      <div className={styles.group}>
        <label className={styles.label}>说明</label>
        <div className={styles.noteBox}>
          在线分析器共用这里的 Prompt。留空时使用内置默认 Prompt。
          可用变量：{PROMPT_VARIABLES.map((item, index) => (
            <span key={item}>
              {index > 0 ? '、' : ' '}
              <code>{item}</code>
            </span>
          ))}
          。
        </div>
      </div>

      <div className={styles.group}>
        <div className={styles.headerRow}>
          <label className={styles.label}>中文 Prompt</label>
          <button className={styles.btnSecondary} onClick={() => resetCurrent('zh')} disabled={loading}>
            恢复默认
          </button>
        </div>
        <textarea
          className={styles.editor}
          rows={16}
          spellCheck={false}
          value={prompt.zh}
          placeholder={PLACEHOLDER_ZH}
          onChange={e => setPrompt(current => ({ ...current, zh: e.target.value }))}
        />
      </div>

      <div className={styles.group}>
        <div className={styles.headerRow}>
          <label className={styles.label}>English Prompt</label>
          <button className={styles.btnSecondary} onClick={() => resetCurrent('en')} disabled={loading}>
            Restore Default
          </button>
        </div>
        <textarea
          className={styles.editor}
          rows={16}
          spellCheck={false}
          value={prompt.en}
          placeholder={PLACEHOLDER_EN}
          onChange={e => setPrompt(current => ({ ...current, en: e.target.value }))}
        />
      </div>

      <div className={styles.actions}>
        <button className={styles.btn} onClick={save} disabled={loading}>保存 Prompt</button>
        <button className={styles.btnSecondary} onClick={load} disabled={loading}>重新加载</button>
      </div>

      {msg && (
        <div className={msgType === 'ok' ? styles.msgOk : styles.msgErr}>{msg}</div>
      )}
    </div>
  )
}
