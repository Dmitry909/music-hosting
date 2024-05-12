import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;

import 'shared_state.dart';
import 'signup_page.dart';
import 'login_page.dart';
import 'upload_track.dart';
import 'search_results_page.dart';

class MainPage extends StatefulWidget {
  @override
  _MainPageState createState() => _MainPageState();
}

class _MainPageState extends State<MainPage> {
  bool _isTokenValid = false;
  String username = "USERNAME INITIAL VALUE";
  TextEditingController _searchController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _checkTokenValidity();
  }

  Future<void> _checkTokenValidity() async {
    username = await getUsername() ?? "FAILED TO GET USERNAME";
    setState(() {
      _isTokenValid = false;
    });
    try {
      final token = (await getToken()) ?? "";
      if (token != "") {
        final response = await http.get(
            Uri.parse('http://localhost:3000/check_token'),
            headers: {'Authorization': token});
        if (response.statusCode == 200) {
          setState(() {
            _isTokenValid = true;
          });
        }
      }
    } catch (e) {}
  }

  Future<void> _logout() async {
    final token = (await getToken())!;
    storeToken(username, "");

    final response = await http.post(
      Uri.parse('http://localhost:3000/logout'),
      headers: {'authorization': token},
    );

    Navigator.pushReplacement(
        context,
        MaterialPageRoute(
          builder: (context) => MainPage(),
        ));
  }

  @override
  Widget build(BuildContext context) {
    return _isTokenValid
        ? _buildWelcomePage(context)
        : _buildLoginSignupPage(context);
  }

  Widget _buildLoginSignupPage(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Main Page'),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            ElevatedButton(
              onPressed: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(builder: (context) => LoginPage()),
                );
              },
              child: Text('Log in'),
            ),
            SizedBox(height: 16),
            ElevatedButton(
              onPressed: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(builder: (context) => SignupPage()),
                );
              },
              child: Text('Sign up'),
            ),
          ],
        ),
      ),
    );
  }

  void _search(BuildContext context) {
    String query = _searchController.text;
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) => SearchResultsPage(query: query),
      ),
    );
  }

  Widget _buildWelcomePage(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Hello, $username!'),
        actions: [
          Padding(
            padding: const EdgeInsets.only(right: 10.0),
            child: Align(
              alignment: Alignment.centerRight,
              child: ElevatedButton(
                onPressed: _logout,
                child: Text('Log out'),
              ),
            ),
          ),
        ],
        bottom: PreferredSize(
          preferredSize: Size.fromHeight(70.0),
          child: Padding(
            padding: EdgeInsets.all(8.0),
            child: TextField(
              controller: _searchController,
              decoration: InputDecoration(
                hintText: 'Search...',
                border:
                    OutlineInputBorder(borderRadius: BorderRadius.circular(8)),
                suffixIcon: IconButton(
                  icon: Icon(Icons.search),
                  onPressed: () => _search(context),
                ),
              ),
              onSubmitted: (value) => _search(context),
            ),
          ),
        ),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            ElevatedButton(
              onPressed: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(builder: (context) => UploadTrackPage()),
                );
              },
              child: Text('Upload track'),
            ),
          ],
        ),
      ),
    );
  }
}
