import { useState } from 'react'
import reactLogo from './assets/react.svg'
import viteLogo from '/vite.svg'
import { getCurrentWindow } from "@tauri-apps/api/window";
import StackCard from './components/StackCard';

function App() {
  return (
    <div>
      <StackCard></StackCard>
    </div>
  );
}

export default App
