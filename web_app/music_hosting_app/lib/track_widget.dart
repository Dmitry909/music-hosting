import 'package:flutter/material.dart';

class TrackWidget extends StatefulWidget {
  final int id;
  final String authorUsername;
  final String name;
  final int cntRates;
  final int sumRates;

  const TrackWidget({
    Key? key,
    required this.id,
    required this.authorUsername,
    required this.name,
    required this.cntRates,
    required this.sumRates,
  }) : super(key: key);

  @override
  _TrackWidgetState createState() => _TrackWidgetState();
}

class _TrackWidgetState extends State<TrackWidget> {
  double _calculateRating() {
    if (widget.cntRates == 0) {
      return 0.0;
    }
    return widget.sumRates / widget.cntRates;
  }

  @override
  Widget build(BuildContext context) {
    return Card(
      elevation: 2,
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: ListTile(
        leading: CircleAvatar(
          child: Text(widget.id.toString()),
        ),
        title: Text(
          widget.name,
          style: const TextStyle(
            fontWeight: FontWeight.bold,
          ),
        ),
        subtitle: Text('by ${widget.authorUsername}'),
        trailing: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.star),
            const SizedBox(width: 4),
            Text(
              _calculateRating().toStringAsFixed(1),
              style: const TextStyle(
                fontWeight: FontWeight.bold,
              ),
            ),
            const SizedBox(width: 4),
            Text('(${widget.cntRates} ratings)'),
          ],
        ),
      ),
    );
  }
}
