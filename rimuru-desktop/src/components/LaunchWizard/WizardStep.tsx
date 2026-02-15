import styles from "./LaunchWizard.module.css";

interface WizardStepProps {
  step: number;
  title: string;
  subtitle?: string;
  children: React.ReactNode;
}

export default function WizardStep({ step, title, subtitle, children }: WizardStepProps) {
  return (
    <div className={styles.step}>
      <div className={styles.stepHeader}>
        <span className={styles.stepBadge}>{step}</span>
        <div>
          <h3 className={styles.stepTitle}>{title}</h3>
          {subtitle && <p className={styles.stepSubtitle}>{subtitle}</p>}
        </div>
      </div>
      <div className={styles.stepContent}>{children}</div>
    </div>
  );
}
