import type { Config } from 'tailwindcss'

const config: Config = {
  darkMode: ['class'],
  content: [
    './index.html',
    './src/**/*.{ts,tsx}',
  ],
  theme: {
    extend: {
      colors: {
        // cluster-ops design tokens (SPEC.md §7)
        background: '#0a0e1a',
        surface: '#0d1117',
        border: '#1e2a3a',
        'text-primary': '#c9d1e9',
        'text-muted': '#4a6a8a',
        accent: '#4a90d9',
        success: '#22c55e',
        warning: '#f59e0b',
        error: '#ef4444',
        'ai-purple': '#7a7adc',
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'Cascadia Code', 'Consolas', 'monospace'],
      },
      fontSize: {
        'xxs': '10px',
        'xs': '11px',
        'sm': '12px',
        'base': '13px',
      },
      borderRadius: {
        'btn': '4px',
        'panel': '6px',
      },
      spacing: {
        'row': '32px',
      },
    },
  },
  plugins: [
    // scrollbar-none utility for the cluster tab strip
    ({ addUtilities }: { addUtilities: (u: Record<string, Record<string, string>>) => void }) => {
      addUtilities({
        '.scrollbar-none': {
          '-ms-overflow-style': 'none',
          'scrollbar-width': 'none',
        },
        '.scrollbar-none::-webkit-scrollbar': {
          display: 'none',
        },
      })
    },
  ],
}

export default config
