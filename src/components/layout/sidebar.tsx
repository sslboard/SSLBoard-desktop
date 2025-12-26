import { NavLink, useLocation } from "react-router-dom";
import { Logo } from "../logo";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from "../ui/sidebar";

export type NavItem = {
  label: string;
  description?: string;
  to: string;
  icon: React.ReactNode;
};

type SidebarProps = {
  items: NavItem[];
  onNavigate?: () => void;
};

export function AppSidebar({ items, onNavigate }: SidebarProps) {
  const location = useLocation();
  const { isMobile, setOpenMobile } = useSidebar();

  const handleNavigate = () => {
    onNavigate?.();
    if (isMobile) {
      setOpenMobile(false);
    }
  };

  return (
    <Sidebar collapsible="offcanvas" className="border-r border-sidebar-border">
      <SidebarHeader className="px-4 py-4">
        <Logo />
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              {items.map((item) => {
                const isActive = location.pathname === item.to;

                return (
                  <SidebarMenuItem key={item.to}>
                    <SidebarMenuButton
                      asChild
                      size="lg"
                      isActive={isActive}
                      onClick={handleNavigate}
                      tooltip={item.label}
                      className="h-auto items-start gap-3 py-3"
                    >
                      <NavLink to={item.to} end={item.to === "/settings"}>
                        <span className="mt-0.5">{item.icon}</span>
                        <span className="flex flex-col gap-0.5">
                          <span className="font-semibold">{item.label}</span>
                          {item.description ? (
                            <span className="text-xs text-muted-foreground">
                              {item.description}
                            </span>
                          ) : null}
                        </span>
                      </NavLink>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                );
              })}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  );
}
