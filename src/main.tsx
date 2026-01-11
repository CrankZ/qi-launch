import ReactDOM from 'react-dom/client';
import AntdProvider from './AntdProvider.tsx';
import App from './App';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <AntdProvider>
    <App />
  </AntdProvider>,
);
