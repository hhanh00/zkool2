import 'package:flutter/material.dart';
import 'package:zkool/utils.dart';

class DisplayPanel extends StatelessWidget {
  final Widget? child;
  const DisplayPanel({this.child, super.key});

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;

    return Container(
      padding: EdgeInsetsGeometry.all(16),
      decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(16),
          gradient: LinearGradient(
            begin: AlignmentGeometry.topCenter,
            end: AlignmentGeometry.bottomCenter,
            colors: [
              cs.surface,
              Color.lerp(cs.surface, cs.primaryContainer, 0.3)!,
            ],
            stops: [0.0, 0.6],
          ),
          boxShadow: [
            BoxShadow(
              color: cs.shadow.withAlpha(30),
              blurRadius: 12,
              offset: Offset(0, 6),
            ),
          ]),
      child: child,
    );
  }
}

class BalanceChip extends StatelessWidget {
  final PoolType pool;
  final String value;
  const BalanceChip(this.pool, this.value, {super.key});

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;

    return Chip(
      label: RichText(
        text: TextSpan(
          children: [
            TextSpan(
              text: "${pool.label} ",
              style: TextStyle(
                fontWeight: FontWeight.w700,
                color: poolTypeColor(pool),
                fontSize: 14,
              ),
            ),
            TextSpan(
              text: value,
              style: TextStyle(
                fontWeight: FontWeight.w400,
                color: cs.primary.withAlpha(180),
                fontSize: 14,
              ),
            ),
          ],
        ),
      ),
      backgroundColor: cs.primary.withAlpha(25),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(20),
        side: BorderSide.none,
      ),
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
    );
  }
}

class TransactionTile extends StatelessWidget {
  final IconData icon;
  final MaterialColor color;
  final String label;
  final BigInt amount;
  final int date;
  final int id;
  final void Function()? onTap;

  const TransactionTile({
    super.key,
    required this.icon,
    required this.color,
    required this.label,
    required this.amount,
    required this.date,
    required this.id,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final d = timeToString(date);
    return ListTile(
      onTap: onTap,
      leading: Container(
        padding: const EdgeInsets.all(10),
        decoration: BoxDecoration(
          color: color.withAlpha(25),
          borderRadius: BorderRadius.circular(12),
        ),
        child: Icon(icon, color: color),
      ),
      title: Text(label),
      subtitle: Text(d),
      trailing: Text(
        zatToString(amount),
        style: TextStyle(
          color: color,
          fontWeight: FontWeight.bold,
          fontSize: 16,
        ),
      ),
    );
  }
}

enum PoolType {
  transparent("T"),
  sapling("S"),
  orchard("O");

  final String label;
  const PoolType(this.label);
}

Color poolTypeColor(PoolType pool) {
  switch (pool) {
    case PoolType.transparent:
      return Colors.red;
    case PoolType.sapling:
      return Colors.orange;
    case PoolType.orchard:
      return Colors.green;
  }
}
