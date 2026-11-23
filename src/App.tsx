import { Features } from '@/components/Features/Features';
import { Footer } from '@/components/Footer/Footer';
import { Hero } from '@/components/Hero/Hero';
import { Install } from '@/components/Install/Install';
import { Navbar } from '@/components/Navbar/Navbar';
import { Scripting } from '@/components/Scripting/Scripting';
import { Sync } from '@/components/Sync/Sync';

function App() {
  return (
    <>
      <Navbar />
      <main id="top">
        <Hero />
        <Sync />
        <Features />
        <Scripting />
        <Install />
      </main>
      <Footer />
    </>
  );
}

export default App;
