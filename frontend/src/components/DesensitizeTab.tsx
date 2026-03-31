import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { DesensitizeConfig, PatternConfig } from '../types'
import styles from './DesensitizeTab.module.css'

export function DesensitizeTab() {
  const [builtinLabels, setBuiltinLabels] = useState<string[]>([])
  const [cfg, setCfg] = useState<DesensitizeConfig | null>(null)
  const [saving, setSaving] = useState(false)
  const [msg, setMsg] = useState<{ text: string; ok: boolean } | null>(null)
  // 新增自定义规则表单
  const [newName, setNewName] = useState('')
  const [newPattern, setNewPattern] = useState('')
  const [patternError, setPatternError] = useState('')

  useEffect(() => {
    Promise.all([
      invoke<string[]>('get_builtin_labels'),
      invoke<DesensitizeConfig>('get_desensitize_config').catch(() => defaultConfig()),
    ]).then(([labels, config]) => {
      setBuiltinLabels(labels)
      setCfg(config)
    })
  }, [])

  const save = async () => {
    if (!cfg) return
    setSaving(true)
    setMsg(null)
    try {
      await invoke('save_desensitize_config', { config: cfg })
      setMsg({ text: '保存成功', ok: true })
    } catch (e) {
      setMsg({ text: String(e), ok: false })
    } finally {
      setSaving(false)
    }
  }

  const toggleEnabled = () =>
    setCfg(c => c ? { ...c, enabled: !c.enabled } : c)

  const toggleBuiltin = (label: string) =>
    setCfg(c => {
      if (!c) return c
      const disabled = c.disabled_builtin.includes(label)
        ? c.disabled_builtin.filter(d => d !== label)
        : [...c.disabled_builtin, label]
      return { ...c, disabled_builtin: disabled }
    })

  const validateAndAddPattern = async () => {
    if (!newName.trim() || !newPattern.trim()) {
      setPatternError('名称和正则不能为空')
      return
    }
    const err: string | null = await invoke('validate_pattern', { pattern: newPattern })
    if (err) {
      setPatternError(`正则无效：${err}`)
      return
    }
    setCfg(c => {
      if (!c) return c
      const p: PatternConfig = { name: newName.trim(), pattern: newPattern.trim(), enabled: true }
      return { ...c, custom_patterns: [...c.custom_patterns, p] }
    })
    setNewName('')
    setNewPattern('')
    setPatternError('')
  }

  const removeCustom = (idx: number) =>
    setCfg(c => c ? { ...c, custom_patterns: c.custom_patterns.filter((_, i) => i !== idx) } : c)

  const toggleCustom = (idx: number) =>
    setCfg(c => {
      if (!c) return c
      const custom_patterns = c.custom_patterns.map((p, i) =>
        i === idx ? { ...p, enabled: !p.enabled } : p
      )
      return { ...c, custom_patterns }
    })

  if (!cfg) return <div className={styles.loading}>加载中…</div>

  return (
    <div className={styles.wrap}>
      {/* 总开关 */}
      <div className={styles.row}>
        <label className={styles.masterLabel}>
          <input type="checkbox" checked={cfg.enabled} onChange={toggleEnabled} />
          <span>启用敏感数据检测</span>
        </label>
        <span className={styles.hint}>关闭后 local 模式不执行脱敏扫描</span>
      </div>

      {/* 内置规则 */}
      <div className={styles.section}>
        <div className={styles.sectionTitle}>内置检测规则</div>
        <div className={styles.grid}>
          {builtinLabels.map(label => {
            const active = !cfg.disabled_builtin.includes(label)
            return (
              <label
                key={label}
                className={`${styles.chip} ${active ? styles.chipOn : styles.chipOff}`}
              >
                <input
                  type="checkbox"
                  checked={active}
                  disabled={!cfg.enabled}
                  onChange={() => toggleBuiltin(label)}
                  className={styles.hidden}
                />
                {label}
              </label>
            )
          })}
        </div>
      </div>

      {/* 自定义规则 */}
      <div className={styles.section}>
        <div className={styles.sectionTitle}>自定义正则规则</div>

        {cfg.custom_patterns.length === 0
          ? <div className={styles.empty}>暂无自定义规则</div>
          : cfg.custom_patterns.map((p, i) => (
            <div key={i} className={styles.customRow}>
              <label className={styles.customCheck}>
                <input
                  type="checkbox"
                  checked={p.enabled}
                  disabled={!cfg.enabled}
                  onChange={() => toggleCustom(i)}
                />
              </label>
              <span className={styles.customName}>{p.name}</span>
              <code className={styles.customPattern}>{p.pattern}</code>
              <button className={styles.btnRemove} onClick={() => removeCustom(i)}>删除</button>
            </div>
          ))
        }

        {/* 添加表单 */}
        <div className={styles.addForm}>
          <input
            className={styles.input}
            placeholder="规则名称（如：手机号）"
            value={newName}
            onChange={e => setNewName(e.target.value)}
          />
          <input
            className={styles.input}
            placeholder="正则表达式（如：1[3-9]\d{9}）"
            value={newPattern}
            onChange={e => { setNewPattern(e.target.value); setPatternError('') }}
          />
          <button className={styles.btnAdd} onClick={validateAndAddPattern}>添加</button>
        </div>
        {patternError && <div className={styles.error}>{patternError}</div>}
      </div>

      {/* 保存 */}
      <button className={styles.btnSave} onClick={save} disabled={saving}>
        {saving ? '保存中…' : '保存配置'}
      </button>
      {msg && <div className={msg.ok ? styles.success : styles.error}>{msg.text}</div>}
    </div>
  )
}

function defaultConfig(): DesensitizeConfig {
  return { enabled: true, disabled_builtin: [], custom_patterns: [] }
}
