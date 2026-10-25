"use client";

import { List, X } from "@phosphor-icons/react";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";

import { NAV } from "../lib/data/site.ts";

/** The header's links: in a row from 768 px, behind a menu button below it. */
export function NavLinks() {
  const path = usePathname();
  const [open, setOpen] = useState(false);
  useEffect(() => setOpen(false), [path]);
  const links = (
    <ul className="m-0 flex list-none flex-col gap-1 p-0 md:flex-row md:items-center md:gap-6">
      {NAV.map((item) => (
        <li key={item.href}>
          <a
            href={item.href}
            aria-current={path === item.href ? "page" : undefined}
            className="block py-2 font-display text-[1.125rem] font-[600] no-underline decoration-2 underline-offset-[0.3em] hover:underline aria-[current=page]:underline"
          >
            {item.label}
          </a>
        </li>
      ))}
    </ul>
  );
  return (
    <>
      <nav aria-label="Main" className="hidden md:block">
        {links}
      </nav>
      <div className="md:hidden">
        <button
          type="button"
          aria-expanded={open}
          aria-controls="menu"
          onClick={() => setOpen((value) => !value)}
          className="inline-flex items-center gap-2 rounded-chart border border-current px-3 py-2 font-display text-[1.0625rem] font-[600]"
        >
          {open ? <X size={18} aria-hidden /> : <List size={18} aria-hidden />}
          Menu
        </button>
        <nav
          id="menu"
          aria-label="Main"
          hidden={!open}
          className="absolute inset-x-0 top-16 z-20 border-b border-rule bg-chart px-4 pb-4"
        >
          {links}
        </nav>
      </div>
    </>
  );
}
