import {
  Box,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemSecondaryAction,
  ListItemText,
  Paper,
  Typography,
  useTheme,
} from "@mui/material";
import {
  ActionId,
  ActionImpl,
  KBarAnimator,
  KBarPortal,
  KBarPositioner,
  KBarProvider,
  KBarResults,
  KBarSearch,
  useMatches,
} from "kbar";
import React, { ReactNode, forwardRef } from "react";

function RenderResults() {
  const { results, rootActionId } = useMatches();

  return (
    <List
      component={KBarResults}
      items={results}
      onRender={({ item, active }) =>
        typeof item === "string" ? (
          <ListItem dense>
            <Typography color="gray" variant="subtitle2">
              {item}
            </Typography>
          </ListItem>
        ) : (
          <ResultItem
            currentRootActionId={rootActionId}
            action={item}
            active={active}
          />
        )
      }
    />
  );
}

interface ResultItemProps {
  action: ActionImpl;
  active: boolean;
  currentRootActionId?: ActionId | null;
}

const ResultItem = forwardRef(
  (
    { action, active, currentRootActionId }: ResultItemProps,
    ref: React.Ref<HTMLDivElement>
  ) => {
    const ancestors = React.useMemo(() => {
      if (!currentRootActionId) return action.ancestors;
      const index = action.ancestors.findIndex(
        (ancestor) => ancestor.id === currentRootActionId
      );
      // +1 removes the currentRootAction; e.g.
      // if we are on the "Set theme" parent action,
      // the UI should not display "Set theme… > Dark"
      // but rather just "Dark"
      return action.ancestors.slice(index + 1);
    }, [action.ancestors, currentRootActionId]);

    return (
      <ListItemButton ref={ref} selected={active}>
        {action.icon && <ListItemIcon>{action.icon}</ListItemIcon>}

        <ListItemText
          primary={
            ancestors.length
              ? `${ancestors.map((a) => a.name).join(" > ")}: ${action.name}`
              : action.name
          }
          secondary={action.subtitle}
        />

        {action.shortcut?.length ? (
          <ListItemSecondaryAction aria-hidden>
            {action.shortcut.map((sc) => (
              <kbd key={sc}>{sc}</kbd>
            ))}
          </ListItemSecondaryAction>
        ) : null}
      </ListItemButton>
    );
  }
);

ResultItem.displayName = "ResultItem";

export function CommandBar({ children }: { children: ReactNode }) {
  const theme = useTheme();

  return (
    <>
      <KBarPortal>
        <KBarPositioner>
          <Paper
            component={KBarAnimator}
            elevation={3}
            sx={{
              maxWidth: "600px",
              width: "100%",
              overflow: "hidden",
            }}
          >
            <Box
              component={KBarSearch}
              sx={{
                width: "100%",
                outline: "none",
                border: "none",
                p: theme.spacing(2),
                color: theme.palette.text.primary,
                background: "transparent",
                ...theme.typography.body1,
              }}
            />
            <RenderResults />
          </Paper>
        </KBarPositioner>
      </KBarPortal>
      {children}
    </>
  );
}

export function CommandBarProvider({ children }: { children: ReactNode }) {
  return <KBarProvider>{children}</KBarProvider>;
}
