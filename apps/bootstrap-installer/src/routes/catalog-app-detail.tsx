interface CatalogAppDetailProps {
  appId?: string
}

export default function CatalogAppDetail({ appId }: CatalogAppDetailProps) {
  return (
    <main data-testid="catalog-app-detail">
      <header>
        <h1>App Detail</h1>
      </header>
      <p className="text-muted-foreground">App: {appId ?? 'unknown'}</p>
    </main>
  )
}
