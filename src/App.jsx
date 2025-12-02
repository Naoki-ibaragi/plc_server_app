import { useState } from 'react'
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
