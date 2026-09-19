import 'package:easy_stepper/easy_stepper.dart';
import 'package:flutter/material.dart';

class LoadingStep {
  final String title;
  final String description;

  const LoadingStep({required this.title, required this.description});
}

/// Read-only progress for an asynchronous operation with sequential stages.
class LoadingSteps extends StatelessWidget {
  final String title;
  final IconData icon;
  final List<LoadingStep> steps;
  final int activeStep;

  const LoadingSteps({
    super.key,
    required this.title,
    required this.icon,
    required this.steps,
    required this.activeStep,
  })  : assert(steps.length > 0),
        assert(activeStep >= 0 && activeStep < steps.length);

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colors = theme.colorScheme;
    return Center(
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 400),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              CircleAvatar(
                radius: 40,
                backgroundColor: colors.primaryContainer,
                child: Icon(icon, size: 40, color: colors.onPrimaryContainer),
              ),
              const SizedBox(height: 24),
              Text(title, textAlign: TextAlign.center, style: theme.textTheme.titleLarge),
              const SizedBox(height: 12),
              Semantics(
                liveRegion: true,
                child: Text(steps[activeStep].description,
                    textAlign: TextAlign.center, style: theme.textTheme.bodyMedium?.copyWith(color: colors.onSurfaceVariant)),
              ),
              const SizedBox(height: 24),
              EasyStepper(
                activeStep: activeStep,
                direction: Axis.vertical,
                verticalAlignment: CrossAxisAlignment.start,
                alignment: AlignmentDirectional.centerStart,
                enableStepTapping: false,
                disableScroll: true,
                showLoadingAnimation: false,
                showStepBorder: false,
                stepRadius: 16,
                internalPadding: 8,
                padding: EdgeInsets.zero,
                activeStepBackgroundColor: colors.surface,
                finishedStepBackgroundColor: colors.primary,
                unreachedStepBackgroundColor: colors.surfaceContainerHighest,
                lineStyle: LineStyle(
                  lineLength: 32,
                  lineType: LineType.normal,
                  lineThickness: 2,
                  defaultLineColor: colors.outlineVariant,
                  finishedLineColor: colors.primary,
                ),
                steps: [
                  for (var i = 0; i < steps.length; i++)
                    EasyStep(
                      customStep: i == activeStep
                          ? SizedBox(width: 24, height: 24, child: CircularProgressIndicator(strokeWidth: 2.5, semanticsLabel: steps[i].title))
                          : Icon(i < activeStep ? Icons.check : Icons.circle,
                              size: i < activeStep ? 20 : 8, color: i < activeStep ? colors.onPrimary : colors.onSurfaceVariant),
                      customTitle: Align(
                        alignment: AlignmentDirectional.centerStart,
                        child: Text(steps[i].title,
                            style: theme.textTheme.bodyMedium?.copyWith(
                                color: i <= activeStep ? colors.onSurface : colors.onSurfaceVariant,
                                fontWeight: i == activeStep ? FontWeight.w600 : FontWeight.normal)),
                      ),
                    ),
                ],
              ),
              const SizedBox(height: 16),
              Text('This may take a few seconds.', style: theme.textTheme.bodySmall?.copyWith(color: colors.onSurfaceVariant)),
            ],
          ),
        ),
      ),
    );
  }
}
