import { useState, useEffect } from "react";

type AnimationState = "entering" | "entered" | "exiting" | "exited";

export function useAnimatedUnmount(isOpen: boolean, duration: number = 200) {
  const [shouldRender, setShouldRender] = useState(isOpen);
  const [animationState, setAnimationState] = useState<AnimationState>(
    isOpen ? "entered" : "exited"
  );

  useEffect(() => {
    if (isOpen) {
      setShouldRender(true);
      setAnimationState("entering");
      const raf = requestAnimationFrame(() => setAnimationState("entered"));
      return () => cancelAnimationFrame(raf);
    } else {
      setAnimationState("exiting");
      const timer = setTimeout(() => {
        setAnimationState("exited");
        setShouldRender(false);
      }, duration);
      return () => clearTimeout(timer);
    }
  }, [isOpen, duration]);

  return { shouldRender, animationState };
}
