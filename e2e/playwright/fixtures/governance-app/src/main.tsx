import React from "react";
import { createRoot } from "react-dom/client";
import { GovernanceFixtureApp } from "./GovernanceFixtureApp";
import "./styles.css";

createRoot(document.getElementById("root") as HTMLElement).render(
	<React.StrictMode>
		<GovernanceFixtureApp />
	</React.StrictMode>,
);
