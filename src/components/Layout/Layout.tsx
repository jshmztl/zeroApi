import * as React from "react";
import { Link, useLocation } from "react-router-dom";
import { Star, History, Globe, FolderOpen, Settings, Upload, Plus } from "lucide-react";
import { useDataStore } from "@/store/dataStore";
import { cn } from "@/lib/utils";
import { Topbar } from "./Topbar";
import { Sidebar } from "./Sidebar";

export function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div className="h-screen w-screen flex flex-col bg-gray-50">
      <Topbar />
      <div className="flex-1 flex overflow-hidden">
        <Sidebar />
        <main className="flex-1 overflow-auto bg-white">{children}</main>
      </div>
    </div>
  );
}
