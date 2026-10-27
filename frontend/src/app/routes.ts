import { Routes } from "@angular/router";
import { Home } from "./pages/home/home";
import { NotFound } from "./pages/not-found/not-found";

const routeConfig: Routes = [
  {
    path: '',
    component: Home,
    title: 'Inari',
  },
  {
    path: '**',
    component: NotFound,
    title: 'Inari - Not Found'
  }
];

export default routeConfig;