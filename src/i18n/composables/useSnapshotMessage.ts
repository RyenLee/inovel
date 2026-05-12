import { useI18n } from 'vue-i18n'

/**
 * 快照消息格式化 Composable
 * 
 * 提供统一的快照消息格式化 API，避免在业务代码中使用语言判断逻辑
 * 
 * @example
 * const { formatSnapshotMessage } = useSnapshotMessage()
 * const message = formatSnapshotMessage('manual', { time: '2024-01-01 12:00:00' })
 */
export function useSnapshotMessage() {
  const { t } = useI18n()

  /**
   * 格式化快照消息
   * @param type - 消息类型：manual（手动）、auto（自动保存）、appClose（应用关闭）
   * @param params - 消息参数：time（时间）、title（章节标题，仅自动保存需要）
   * @returns 格式化后的消息
   */
  const formatSnapshotMessage = (
    type: 'manual' | 'auto' | 'appClose',
    params: { title?: string; time: string }
  ): string => {
    const keyMap = {
      manual: 'manualSnapshotMessage',
      auto: 'autoSaveMessage',
      appClose: 'appCloseMessage',
    }
    const key = `editor.snapshot.${keyMap[type]}`
    return t(key, params)
  }

  return {
    formatSnapshotMessage,
  }
}
