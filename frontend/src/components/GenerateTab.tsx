import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import type { ReportDto } from '../types'
import styles from './GenerateTab.module.css'

type ReportType = 'daily' | 'weekly' | 'custom' | 'serilog'

interface Props {
  onReportReady: (report: ReportDto) => void
}

export function GenerateTab({ onReportReady }: Props) {
  const [rtype, setRtype] = useState<ReportType>('daily')
  const [customFrom, setCustomFrom] = useState('')
  const [customTo, setCustomTo] = useState('')
  const [serilogPath, setSerilogPath] = useState('')
  const [serilogFrom, setSerilogFrom] = useState('')
  const [serilogTo, setSerilogTo] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  const pickFile = async () => {
    const selected = await openDialog({ multiple: false, directory: false })
    if (selected) setSerilogPath(selected as string)
  }

  const run = async () => {
    setError('')
    setLoading(true)
    try {
      let report: ReportDto
      if (rtype === 'daily') {
        report = await invoke('generate_daily_report')
      } else if (rtype === 'weekly') {
        report = await invoke('generate_weekly_report')
      } else if (rtype === 'custom') {
        if (!customFrom || !customTo) throw new Error('请选择开始和结束日期')
        report = await invoke('generate_custom_report', { from: customFrom, to: customTo })
      } else {
        if (!serilogPath) throw new Error('请先选择日志文件路径')
        report = await invoke('generate_serilog_report', {
          path: serilogPath,
          from: serilogFrom || null,
          to: serilogTo || null,
        })
      }
      onReportReady(report)
    } catch (e) {
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }

  const TYPES: { value: ReportType; label: string }[] = [
    { value: 'daily',   label: '昨日报告' },
    { value: 'weekly',  label: '周报告（7天）' },
    { value: 'custom',  label: '自定义范围' },
    { value: 'serilog', label: 'Serilog 文件' },
  ]

  return (
    <div className={styles.wrap}>
      {/* 报告类型 */}
      <div className={styles.group}>
        <label className={styles.label}>报告类型</label>
        <div className={styles.radioRow}>
          {TYPES.map(t => (
            <label key={t.value} className={`${styles.chip} ${rtype === t.value ? styles.chipActive : ''}`}>
              <input
                type="radio"
                name="rtype"
                value={t.value}
                checked={rtype === t.value}
                onChange={() => setRtype(t.value)}
                className={styles.hidden}
              />
              {t.label}
            </label>
          ))}
        </div>
      </div>

      {/* 自定义范围 */}
      {rtype === 'custom' && (
        <div className={styles.group}>
          <label className={styles.label}>日期范围</label>
          <div className={styles.dateRow}>
            <div className={styles.dateField}>
              <span className={styles.sub}>开始日期</span>
              <input type="date" value={customFrom} onChange={e => setCustomFrom(e.target.value)} className={styles.dateInput} />
            </div>
            <span className={styles.dateSep}>—</span>
            <div className={styles.dateField}>
              <span className={styles.sub}>结束日期</span>
              <input type="date" value={customTo} onChange={e => setCustomTo(e.target.value)} className={styles.dateInput} />
            </div>
          </div>
        </div>
      )}

      {/* Serilog 选项 */}
      {rtype === 'serilog' && (
        <div className={styles.group}>
          <label className={styles.label}>日志路径</label>
          <div className={styles.fileRow}>
            <button className={styles.btnSecondary} onClick={pickFile}>选择文件</button>
            <span className={styles.filepath}>{serilogPath || '未选择'}</span>
          </div>
          <div className={styles.dateRow} style={{ marginTop: 12 }}>
            <div className={styles.dateField}>
              <span className={styles.sub}>开始日期（可选）</span>
              <input type="date" value={serilogFrom} onChange={e => setSerilogFrom(e.target.value)} className={styles.dateInput} />
            </div>
            <span className={styles.dateSep}>—</span>
            <div className={styles.dateField}>
              <span className={styles.sub}>结束日期（可选）</span>
              <input type="date" value={serilogTo} onChange={e => setSerilogTo(e.target.value)} className={styles.dateInput} />
            </div>
          </div>
        </div>
      )}

      {/* 运行按钮 */}
      <button className={styles.btn} onClick={run} disabled={loading}>
        {loading ? '分析中…' : '生成报告'}
      </button>

      {/* 进度 */}
      {loading && (
        <div className={styles.progress}>
          <span className={styles.spinner} />
          正在分析日志，请稍候…
        </div>
      )}

      {/* 错误 */}
      {error && <div className={styles.error}>{error}</div>}
    </div>
  )
}
