import AdminApp from './AdminApp.svelte';
import './admin.css';

new AdminApp({
  target: document.getElementById('admin-app') as HTMLElement
});
