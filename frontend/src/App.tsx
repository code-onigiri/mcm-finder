import React, { useEffect, useState } from 'react';
import './App.css';
import ModDetailPage from './pages/ModDetailPage';
import SearchPage from './pages/SearchPage';

type Route = { page: 'search' } | { page: 'detail'; modId: string };

function parseRoute(): Route {
  const hash = window.location.hash.replace(/^#/, '');
  if (hash.startsWith('/mods/')) {
    const modId = hash.replace('/mods/', '').trim();
    if (modId) {
      return { page: 'detail', modId };
    }
  }
  return { page: 'search' };
}

function App() {
  const [route, setRoute] = useState<Route>(parseRoute());

  useEffect(() => {
    const onHashChange = () => setRoute(parseRoute());
    window.addEventListener('hashchange', onHashChange);
    return () => window.removeEventListener('hashchange', onHashChange);
  }, []);

  return (
    <div className="App">
      {route.page === 'detail' ? <ModDetailPage modId={route.modId} /> : <SearchPage />}
    </div>
  );
}

export default App;
