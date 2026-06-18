import { cn } from '@/lib/utils'

const assetPath = (path: string) => `${import.meta.env.BASE_URL}${path.replace(/^\/+/, '')}`

// YClaw brand color used for the tile tint. Matches the trio PNG's flat
// red-orange (#F04E23) so the mark and its backdrop feel like one piece.
const BRAND = '#F04E23'

// Brand badge: YClaw trio mark on a softly-tinted tile. The PNG is
// transparent; the tile carries a brand-orange wash (~8% over the page
// background, with a matching low-opacity inner ring) so the mark has
// visual weight on either light or dark themes without going pure white.
export function BrandMark({ className, ...props }: React.ComponentProps<'span'>) {
  return (
    <span
      className={cn(
        'inline-flex size-14 shrink-0 items-center justify-center overflow-hidden rounded-lg',
        // Tailwind 4 arbitrary value: brand orange 8% over transparent.
        // The transparency lets the page bg show through, so this reads
        // as a warm tint in light mode and a deep ember in dark mode.
        'bg-[color-mix(in_srgb,var(--yclaw-brand)_8%,transparent)]',
        // Inner ring for definition on neutral surfaces (matches the
        // 8% wash so it stays subtle).
        'ring-1 ring-inset ring-[color-mix(in_srgb,var(--yclaw-brand)_18%,transparent)]',
        className
      )}
      style={{ ['--yclaw-brand' as string]: BRAND }}
      {...props}
    >
      <img alt="" className="size-full object-contain" src={assetPath('nous-girl.png')} />
    </span>
  )
}
