import {
  ShieldCheck,
  Wand2,
  Radar,
  Settings as SettingsIcon,
  Globe,
} from "lucide-react";
import { Navigate, Route, Routes } from "react-router-dom";
import { AppShell } from "./components/layout/app-shell";
import { ThemeProvider } from "./components/theme-provider";
import { CertificatesPage } from "./pages/Certificates";
import { IssuePage } from "./pages/Issue";
import { DiscoverPage } from "./pages/Discover";
import { SettingsPage } from "./pages/Settings";
import { DnsProvidersPage } from "./pages/settings/DnsProviders";
import type { NavItem } from "./components/layout/sidebar";

const navItems: NavItem[] = [
  {
    label: "Certificates",
    description: "Inventory and lifecycle insights.",
    to: "/certificates",
    icon: <ShieldCheck className="h-5 w-5" />,
  },
  {
    label: "Issue",
    description: "Start issuance and validation.",
    to: "/issue",
    icon: <Wand2 className="h-5 w-5" />,
  },
  {
    label: "Discover",
    description: "Find certs across infrastructure.",
    to: "/discover",
    icon: <Radar className="h-5 w-5" />,
  },
  {
    label: "Settings",
    description: "Configure providers and policies.",
    to: "/settings",
    icon: <SettingsIcon className="h-5 w-5" />,
  },
  {
    label: "DNS Providers",
    description: "Configure automatic DNS providers.",
    to: "/settings/dns-providers",
    icon: <Globe className="h-5 w-5" />,
  },
];

function App() {
  return (
    <ThemeProvider>
      <AppShell navItems={navItems}>
        <Routes>
          <Route path="/" element={<Navigate to="/certificates" replace />} />
          <Route path="/certificates" element={<CertificatesPage />} />
          <Route path="/issue" element={<IssuePage />} />
          <Route path="/discover" element={<DiscoverPage />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="/settings/dns-providers" element={<DnsProvidersPage />} />
        </Routes>
      </AppShell>
    </ThemeProvider>
  );
}

export default App;
