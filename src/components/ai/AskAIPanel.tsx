import { useEffect, useRef, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { Send, X } from 'lucide-react'

// ── Types ─────────────────────────────────────────────────────────────────────

interface Message {
  role: 'user' | 'assistant'
  content: string
  timestamp: Date
}

interface AskAIPanelProps {
  rawOutput: string
  mode: 'describe' | 'logs' | 'security' | 'network-scan' | 'rbac-scan'
  onClose: () => void
}

// ── AskAIPanel ────────────────────────────────────────────────────────────────

export function AskAIPanel({ rawOutput, mode: _mode, onClose }: AskAIPanelProps) {
  const [providerLabel, setProviderLabel] = useState('Ask AI')
  const [messages,      setMessages]      = useState<Message[]>([])
  const [input,         setInput]         = useState('')
  const [isLoading,     setIsLoading]     = useState(false)

  const messagesEndRef  = useRef<HTMLDivElement>(null)
  const inputRef        = useRef<HTMLTextAreaElement>(null)
  const streamingMsgRef = useRef('')
  const unlistensRef    = useRef<(() => void)[]>([])

  // ── Provider label ────────────────────────────────────────────────────────

  useEffect(() => {
    invoke<string>('get_ai_provider_name').then(setProviderLabel).catch(() => {})
  }, [])

  // ── Auto-scroll ───────────────────────────────────────────────────────────

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  // ── Cleanup listeners on unmount ─────────────────────────────────────────

  useEffect(() => () => {
    unlistensRef.current.forEach((fn) => fn())
  }, [])

  // ── Send ──────────────────────────────────────────────────────────────────

  const send = async () => {
    const question = input.trim()
    if (!question || isLoading) return

    setInput('')
    setIsLoading(true)

    const userMsg: Message = { role: 'user', content: question, timestamp: new Date() }
    setMessages((prev) => [...prev, userMsg])

    // Placeholder for the streaming AI reply
    const aiMsg: Message = { role: 'assistant', content: '', timestamp: new Date() }
    setMessages((prev) => [...prev, aiMsg])
    streamingMsgRef.current = ''

    // Build conversation history (exclude the empty placeholder we just added)
    const history = messages.map((m) => ({ role: m.role, content: m.content }))

    try {
      // Clean up any previous listeners
      unlistensRef.current.forEach((fn) => fn())

      const uls = await Promise.all([
        listen<string>('ask-ai-stream', (e) => {
          streamingMsgRef.current += e.payload
          setMessages((prev) => {
            const updated = [...prev]
            updated[updated.length - 1] = {
              ...updated[updated.length - 1],
              content: streamingMsgRef.current,
            }
            return updated
          })
        }),
        listen<string>('ask-ai-done', (e) => {
          unlistensRef.current.forEach((fn) => fn())
          unlistensRef.current = []
          setMessages((prev) => {
            const updated = [...prev]
            updated[updated.length - 1] = {
              ...updated[updated.length - 1],
              content: e.payload,
            }
            return updated
          })
          setIsLoading(false)
        }),
      ])
      unlistensRef.current = uls

      await invoke('ask_ai', {
        rawOutput,
        messages: history,
        question,
      })
    } catch (err) {
      unlistensRef.current.forEach((fn) => fn())
      unlistensRef.current = []
      setMessages((prev) => {
        const updated = [...prev]
        updated[updated.length - 1] = {
          ...updated[updated.length - 1],
          content: `Error: ${String(err)}`,
        }
        return updated
      })
      setIsLoading(false)
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      send()
    }
  }

  const fmt = (d: Date) =>
    d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })

  // ── Render ────────────────────────────────────────────────────────────────

  return (
    <div
      style={{
        position: 'fixed',
        bottom: 0,
        right: 0,
        width: 380,
        height: '55vh',
        background: '#080c14',
        border: '1px solid #1e2a3a',
        borderBottom: 'none',
        borderRadius: '8px 8px 0 0',
        boxShadow: '0 -4px 24px rgba(0,0,0,0.4)',
        zIndex: 1000,
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      {/* Header */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '10px 14px',
          borderBottom: '1px solid #1e2a3a',
          flexShrink: 0,
        }}
      >
        <span style={{ fontSize: 13, fontWeight: 600, color: '#7a7adc' }}>
          💬 {providerLabel}
        </span>
        <button
          onClick={onClose}
          style={{
            background: 'none',
            border: 'none',
            cursor: 'pointer',
            color: '#6b7280',
            display: 'flex',
            alignItems: 'center',
            padding: 2,
          }}
          title="Close"
        >
          <X size={14} />
        </button>
      </div>

      {/* Messages */}
      <div
        style={{
          flex: 1,
          overflowY: 'auto',
          padding: '12px 14px',
          display: 'flex',
          flexDirection: 'column',
          gap: 10,
        }}
      >
        {messages.length === 0 && (
          <p style={{ fontSize: 12, color: '#4b5563', margin: 0 }}>
            Ask anything about the kubectl output above.
          </p>
        )}

        {messages.map((msg, i) => (
          <div
            key={i}
            style={{
              display: 'flex',
              flexDirection: 'column',
              alignItems: msg.role === 'user' ? 'flex-end' : 'flex-start',
            }}
          >
            <div
              style={{
                maxWidth: '85%',
                padding: '7px 11px',
                borderRadius: 8,
                fontSize: 12,
                lineHeight: 1.5,
                background: msg.role === 'user' ? '#1a2a4a' : '#0d1117',
                color:      msg.role === 'user' ? '#ffffff'  : '#c9d1e9',
                whiteSpace: 'pre-wrap',
                wordBreak: 'break-word',
              }}
            >
              {msg.content || (
                isLoading && i === messages.length - 1 ? (
                  <span style={{ display: 'flex', gap: 4, alignItems: 'center' }}>
                    {[0, 1, 2].map((dot) => (
                      <span
                        key={dot}
                        style={{
                          width: 6,
                          height: 6,
                          borderRadius: '50%',
                          background: '#7a7adc',
                          display: 'inline-block',
                          animation: 'bounce 1s infinite',
                          animationDelay: `${dot * 0.15}s`,
                        }}
                      />
                    ))}
                  </span>
                ) : null
              )}
            </div>
            <span style={{ fontSize: 10, color: '#374151', marginTop: 2 }}>
              {fmt(msg.timestamp)}
            </span>
          </div>
        ))}

        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div
        style={{
          padding: '10px 14px',
          borderTop: '1px solid #1e2a3a',
          display: 'flex',
          gap: 8,
          alignItems: 'flex-end',
          flexShrink: 0,
        }}
      >
        <textarea
          ref={inputRef}
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Type your question…"
          rows={1}
          style={{
            flex: 1,
            background: '#0d1117',
            border: '1px solid #1e2a3a',
            borderRadius: 6,
            color: '#c9d1e9',
            fontSize: 12,
            padding: '6px 10px',
            resize: 'none',
            outline: 'none',
            fontFamily: 'inherit',
            lineHeight: 1.5,
          }}
        />
        <button
          onClick={send}
          disabled={!input.trim() || isLoading}
          style={{
            background: input.trim() && !isLoading ? '#7a7adc' : '#1e2a3a',
            border: 'none',
            borderRadius: 6,
            color: '#ffffff',
            cursor: input.trim() && !isLoading ? 'pointer' : 'not-allowed',
            padding: '6px 10px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            flexShrink: 0,
            transition: 'background 0.15s',
          }}
          title="Send (Enter)"
        >
          <Send size={13} />
        </button>
      </div>
    </div>
  )
}
