import { useStore } from '@nanostores/react'
import { useEffect } from 'react'
import { $route, $currentAppId, $apps, initialize, applyPendingUpdate } from './store'
import { fetchLaunchMode, resolveInitialRoute } from './lib/launcher-mode'
import Welcome from './routes/welcome'
import Progress from './routes/progress'
import Success from './routes/success'
import Failure from './routes/failure'
import { Home } from './routes/home'
import { AppDetail } from './routes/app-detail'
import Settings from './routes/settings'
import Catalog from './routes/catalog'
import CatalogAppDetail from './routes/catalog-app-detail'

export default function App() {
  const route = useStore($route)
  const currentAppId = useStore($currentAppId)
  const apps = useStore($apps)

  useEffect(() => {
    void initialize()
    void fetchLaunchMode().then((mode) => {
      const initial = resolveInitialRoute(mode)
      $route.set(initial)
    })
  }, [])

  function handleSelectApp(appId: string) {
    $currentAppId.set(appId)
    $route.set('app-detail')
  }

  function handleHome() {
    $route.set('home')
  }

  function handleInstallAllPending() {
    const ready = Object.values(apps).filter((a) => a.pending?.status === 'ready')
    for (const app of ready) {
      void applyPendingUpdate(app.descriptor.id)
    }
  }

  if ((route as string) === 'catalog') {
    return (
      <div className="relative flex h-full flex-col overflow-hidden bg-background text-foreground">
        <main className="relative z-10 flex flex-1 flex-col overflow-hidden">
          <Catalog />
        </main>
      </div>
    )
  }

  if ((route as string) === 'catalog-app-detail') {
    return (
      <div className="relative flex h-full flex-col overflow-hidden bg-background text-foreground">
        <main className="relative z-10 flex flex-1 flex-col overflow-hidden">
          <CatalogAppDetail appId={currentAppId ?? undefined} />
        </main>
      </div>
    )
  }

  return (
    <div className="relative flex h-full flex-col overflow-hidden bg-background text-foreground">
      <main className="relative z-10 flex flex-1 flex-col overflow-hidden">
        {route === 'welcome' && <Welcome />}
        {route === 'home' && (
          <Home
            apps={Object.values(apps)}
            onSelect={handleSelectApp}
            onInstallAllPending={handleInstallAllPending}
          />
        )}
        {route === 'app-detail' && currentAppId && (
          <AppDetail
            app={apps[currentAppId]}
            onAction={() => {}}
            onBack={handleHome}
          />
        )}
        {route === 'settings' && <Settings />}
        {route === 'progress' && currentAppId && <Progress appId={currentAppId} />}
        {route === 'success' && currentAppId && <Success appId={currentAppId} />}
        {route === 'failure' && currentAppId && <Failure appId={currentAppId} />}
      </main>
    </div>
  )
}
